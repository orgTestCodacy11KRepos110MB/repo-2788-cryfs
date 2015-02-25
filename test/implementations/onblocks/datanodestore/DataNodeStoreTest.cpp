#include "../../../../implementations/onblocks/datanodestore/DataInnerNode.h"
#include "../../../../implementations/onblocks/datanodestore/DataLeafNode.h"
#include "../../../../implementations/onblocks/datanodestore/DataNode.h"
#include "../../../../implementations/onblocks/datanodestore/DataNodeStore.h"
#include <google/gtest/gtest.h>

#include "messmer/blockstore/implementations/testfake/FakeBlockStore.h"
#include "messmer/blockstore/implementations/testfake/FakeBlock.h"
#include "messmer/blobstore/implementations/onblocks/BlobStoreOnBlocks.h"

using ::testing::Test;
using std::unique_ptr;
using std::make_unique;
using std::string;

using blockstore::BlockStore;
using blockstore::testfake::FakeBlockStore;
using blockstore::Key;
using namespace blobstore;
using namespace blobstore::onblocks;
using namespace blobstore::onblocks::datanodestore;

class DataNodeStoreTest: public Test {
public:
  static constexpr uint32_t BLOCKSIZE_BYTES = 1024;

  unique_ptr<BlockStore> _blockStore = make_unique<FakeBlockStore>();
  BlockStore *blockStore = _blockStore.get();
  unique_ptr<DataNodeStore> nodeStore = make_unique<DataNodeStore>(std::move(_blockStore), BLOCKSIZE_BYTES);
};

constexpr uint32_t DataNodeStoreTest::BLOCKSIZE_BYTES;

#define EXPECT_IS_PTR_TYPE(Type, ptr) EXPECT_NE(nullptr, dynamic_cast<Type*>(ptr)) << "Given pointer cannot be cast to the given type"

TEST_F(DataNodeStoreTest, CreateLeafNodeCreatesLeafNode) {
  auto node = nodeStore->createNewLeafNode();
  EXPECT_IS_PTR_TYPE(DataLeafNode, node.get());
}

TEST_F(DataNodeStoreTest, CreateInnerNodeCreatesInnerNode) {
  auto leaf = nodeStore->createNewLeafNode();

  auto node = nodeStore->createNewInnerNode(*leaf);
  EXPECT_IS_PTR_TYPE(DataInnerNode, node.get());
}

TEST_F(DataNodeStoreTest, LeafNodeIsRecognizedAfterStoreAndLoad) {
  Key key = nodeStore->createNewLeafNode()->key();

  auto loaded_node = nodeStore->load(key);

  EXPECT_IS_PTR_TYPE(DataLeafNode, loaded_node.get());
}

TEST_F(DataNodeStoreTest, InnerNodeWithDepth1IsRecognizedAfterStoreAndLoad) {
  auto leaf = nodeStore->createNewLeafNode();
  Key key = nodeStore->createNewInnerNode(*leaf)->key();

  auto loaded_node = nodeStore->load(key);

  EXPECT_IS_PTR_TYPE(DataInnerNode, loaded_node.get());
}

TEST_F(DataNodeStoreTest, InnerNodeWithDepth2IsRecognizedAfterStoreAndLoad) {
  auto leaf = nodeStore->createNewLeafNode();
  auto inner = nodeStore->createNewInnerNode(*leaf);
  Key key = nodeStore->createNewInnerNode(*inner)->key();

  auto loaded_node = nodeStore->load(key);

  EXPECT_IS_PTR_TYPE(DataInnerNode, loaded_node.get());
}

TEST_F(DataNodeStoreTest, DataNodeCrashesOnLoadIfDepthIsTooHigh) {
  auto block = blockStore->create(BLOCKSIZE_BYTES);
  Key key = block->key();
  {
    DataNodeView view(std::move(block));
    *view.Depth() = DataNodeStore::MAX_DEPTH + 1;
  }

  EXPECT_ANY_THROW(
    nodeStore->load(key)
  );
}

TEST_F(DataNodeStoreTest, CreatedInnerNodeIsInitialized) {
  auto leaf = nodeStore->createNewLeafNode();
  auto node = nodeStore->createNewInnerNode(*leaf);
  EXPECT_EQ(1u, node->numChildren());
  EXPECT_EQ(leaf->key(), node->getChild(0)->key());
}

TEST_F(DataNodeStoreTest, CreatedLeafNodeIsInitialized) {
  auto leaf = nodeStore->createNewLeafNode();
  EXPECT_EQ(0u, leaf->numBytes());
}

TEST_F(DataNodeStoreTest, NodeIsNotLoadableAfterDeleting) {
  auto nodekey = nodeStore->createNewLeafNode()->key();
  auto node = nodeStore->load(nodekey);
  EXPECT_NE(nullptr, node.get());
  nodeStore->remove(std::move(node));
  EXPECT_EQ(nullptr, nodeStore->load(nodekey).get());
}

TEST_F(DataNodeStoreTest, NumNodesIsCorrectOnEmptyNodestore) {
  EXPECT_EQ(0, nodeStore->numNodes());
}

TEST_F(DataNodeStoreTest, NumNodesIsCorrectAfterAddingOneLeafNode) {
  nodeStore->createNewLeafNode();
  EXPECT_EQ(1, nodeStore->numNodes());
}

TEST_F(DataNodeStoreTest, NumNodesIsCorrectAfterRemovingTheLastNode) {
  auto leaf = nodeStore->createNewLeafNode();
  nodeStore->remove(std::move(leaf));
  EXPECT_EQ(0, nodeStore->numNodes());
}

TEST_F(DataNodeStoreTest, NumNodesIsCorrectAfterAddingTwoNodes) {
  auto leaf = nodeStore->createNewLeafNode();
  auto node = nodeStore->createNewInnerNode(*leaf);
  EXPECT_EQ(2, nodeStore->numNodes());
}

TEST_F(DataNodeStoreTest, NumNodesIsCorrectAfterRemovingANode) {
  auto leaf = nodeStore->createNewLeafNode();
  auto node = nodeStore->createNewInnerNode(*leaf);
  nodeStore->remove(std::move(node));
  EXPECT_EQ(1, nodeStore->numNodes());
}
