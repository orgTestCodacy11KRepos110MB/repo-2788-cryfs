#include <unistd.h>
#include <sys/types.h>
#include <sys/wait.h>
#include "FuseThread.h"
#include <csignal>
#include <messmer/cpp-utils/assert/assert.h>
#include "../../fuse/Fuse.h"

using boost::thread;
using boost::chrono::seconds;
using std::string;

using fspp::fuse::Fuse;

FuseThread::FuseThread(Fuse *fuse)
  :_fuse(fuse), _child() {
}

void FuseThread::start(int argc, char *argv[]) {
  _child = thread([this, argc, argv] () {
    _fuse->run(argc, argv);
  });
  //Wait until it is running (busy waiting is simple and doesn't hurt much here)
  while(!_fuse->running()) {}
}

void FuseThread::stop() {
  pthread_kill(_child.native_handle(), SIGINT);
  bool thread_stopped = _child.try_join_for(seconds(5));
  ASSERT(thread_stopped, "FuseThread could not be stopped");
  //Wait until it is properly shutdown (busy waiting is simple and doesn't hurt much here)
  while (_fuse->running()) {}
}