#pragma once
#ifndef MESSMER_CRYFS_FILESYSTEM_CRYNODE_H_
#define MESSMER_CRYFS_FILESYSTEM_CRYNODE_H_

#include <fspp/fs_interface/Node.h>
#include <cpp-utils/macros.h>
#include <fspp/fs_interface/Dir.h>
#include "cryfs/impl/filesystem/rustfsblobstore/RustDirBlob.h"
#include "CryDevice.h"

namespace cryfs {

class CryNode: public fspp::Node {
public:
  virtual ~CryNode();

  // TODO grandparent is only needed to set the timestamps of the parent directory on rename and remove. Delete grandparent parameter once we store timestamps in the blob itself instead of in the directory listing.
  CryNode(CryDevice *device, boost::optional<cpputils::unique_ref<fsblobstore::rust::RustDirBlob>> parent, boost::optional<cpputils::unique_ref<fsblobstore::rust::RustDirBlob>> grandparent, const blockstore::BlockId &blockId);

  void access(int mask) const override;
  stat_info stat() const override;
  void chmod(fspp::mode_t mode) override;
  void chown(fspp::uid_t uid, fspp::gid_t gid) override;
  void rename(const boost::filesystem::path &to) override;
  void utimens(timespec lastAccessTime, timespec lastModificationTime) override;

  // used in test cases
  bool checkParentPointer();

protected:
  CryNode();

  CryDevice *device();
  const CryDevice *device() const;
  const blockstore::BlockId &blockId() const;
  cpputils::unique_ref<fsblobstore::rust::RustFsBlob> LoadBlob() const;
  bool isRootDir() const;
  std::shared_ptr<const fsblobstore::rust::RustDirBlob> parent() const;
  std::shared_ptr<fsblobstore::rust::RustDirBlob> parent();
  boost::optional<fsblobstore::rust::RustDirBlob*> grandparent();
  fspp::TimestampUpdateBehavior timestampUpdateBehavior() const;

  virtual fspp::Dir::EntryType getType() const = 0;

  void removeNode();

private:
  void _updateParentModificationTimestamp();
  void _updateTargetDirModificationTimestamp(const fsblobstore::rust::RustDirBlob &targetDir, boost::optional<cpputils::unique_ref<fsblobstore::rust::RustDirBlob>> targetDirParent);

  CryDevice *_device;
  boost::optional<std::shared_ptr<fsblobstore::rust::RustDirBlob>> _parent;
  boost::optional<cpputils::unique_ref<fsblobstore::rust::RustDirBlob>> _grandparent;
  blockstore::BlockId _blockId;

  DISALLOW_COPY_AND_ASSIGN(CryNode);
};

}

#endif
