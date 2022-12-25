#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CRYOPENFILE_H_
#define MESSMER_CRYFS_FILESYSTEM_CRYOPENFILE_H_

#include <fspp/fs_interface/OpenFile.h>
#include "cryfs/impl/filesystem/rustfsblobstore/RustFileBlob.h"
#include "cryfs/impl/filesystem/rustfsblobstore/RustDirBlob.h"

namespace cryfs {
class CryDevice;

class CryOpenFile final: public fspp::OpenFile {
public:
  explicit CryOpenFile(const CryDevice *device, std::shared_ptr<fsblobstore::rust::RustDirBlob> parent, cpputils::unique_ref<fsblobstore::rust::RustFileBlob> fileBlob);
  ~CryOpenFile();

  stat_info stat() const override;
  void truncate(fspp::num_bytes_t size) const override;
  fspp::num_bytes_t read(void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) const override;
  void write(const void *buf, fspp::num_bytes_t count, fspp::num_bytes_t offset) override;
  void flush() override;
  void fsync() override;
  void fdatasync() override;
  fspp::TimestampUpdateBehavior timestampUpdateBehavior() const;

private:
  const CryDevice *_device;
  std::shared_ptr<fsblobstore::rust::RustDirBlob> _parent;
  cpputils::unique_ref<fsblobstore::rust::RustFileBlob> _fileBlob;

  DISALLOW_COPY_AND_ASSIGN(CryOpenFile);
};

}

#endif
