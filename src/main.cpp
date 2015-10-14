#include <messmer/blockstore/implementations/ondisk/OnDiskBlockStore.h>
#include <messmer/blockstore/implementations/inmemory/InMemoryBlockStore.h>
#include <messmer/blockstore/implementations/inmemory/InMemoryBlock.h>
#include <cmath>
#include <cstdio>
#include <cstdlib>
#include <messmer/cpp-utils/assert/backtrace.h>

#include "messmer/fspp/fuse/Fuse.h"
#include "messmer/fspp/impl/FilesystemImpl.h"
#include "filesystem/CryDevice.h"
#include "config/CryConfigLoader.h"
#include "program_options/Parser.h"

#include <gitversion/version.h>

using namespace cryfs;
namespace bf = boost::filesystem;

using blockstore::ondisk::OnDiskBlockStore;
using blockstore::inmemory::InMemoryBlockStore;
using program_options::ProgramOptions;

using cpputils::make_unique_ref;
using std::cout;
using std::endl;
using std::vector;

//TODO Support files > 4GB
//TODO cryfs process doesn't seem to react to "kill". Needs "kill -9". Why? Furthermore, calling "fusermount -u" unmounts the fs, but the cryfs process keeps running. Why?
//TODO Improve parallelity.
//TODO Seems to deadlock in bonnie++ second run (in the create files sequentially) - maybe also in a later run or different step?

void showVersion() {
    cout << "CryFS Version " << version::VERSION_STRING << endl;
    if (version::IS_DEV_VERSION) {
        cout << "WARNING! This is a development version based on git commit " << version::GIT_COMMIT_ID <<
        ". Please do not use in production!" << endl;
    } else if (!version::IS_STABLE_VERSION) {
        cout << "WARNING! This is an experimental version. Please backup your data frequently!" << endl;
    }
#ifndef NDEBUG
    cout << "WARNING! This is a debug build. Performance might be slow." << endl;
#endif
    cout << endl;
}

void runFilesystem(const ProgramOptions &options) {
    auto config = CryConfigLoader().loadOrCreate(bf::path(options.configFile()));
    auto blockStore = make_unique_ref<OnDiskBlockStore>(bf::path(options.baseDir()));
    CryDevice device(std::move(config), std::move(blockStore));
    fspp::FilesystemImpl fsimpl(&device);
    fspp::fuse::Fuse fuse(&fsimpl);

    vector<char*> fuseOptions = options.fuseOptions();
    fuse.run(fuseOptions.size(), fuseOptions.data());
}

int main(int argc, char *argv[]) {
    cpputils::showBacktraceOnSigSegv();

    showVersion();
    ProgramOptions options = program_options::Parser(argc, argv).parse();
    runFilesystem(options);
    return 0;
}