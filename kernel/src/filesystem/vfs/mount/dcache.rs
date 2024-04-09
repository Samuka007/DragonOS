//! 文件系统目录项缓存
//! Todo: 更改使用的哈希
use alloc::{
    collections::LinkedList,
    string::String,
    sync::{Arc, Weak},
};

use hashbrown::HashSet;
use path_base::Path;

use crate::{
    filesystem::vfs::{
        utils::{Key, Keyable},
        IndexNode,
    },
    libs::rwlock::RwLock,
};

use super::MountFSInode;

#[derive(Debug)]
pub struct DCache {
    table: RwLock<HashSet<Key<Resource>>>,
    lru_list: RwLock<LinkedList<Arc<MountFSInode>>>,
    max_size: usize,
}

#[allow(dead_code)]
const DEFAULT_MEMORY_SIZE: usize = 1024 /* K */ * 1024 /* Byte */;

// pub trait DCache {
//     /// 创建一个新的目录项缓存
//     fn new(mem_size: Option<usize>) -> Self;
//     /// 缓存目录项
//     fn put(&self, src: &Arc<MountFSInode>);
//     /// 清除失效目录项，返回清除的数量（可能的usize话）
//     fn clean(&self, num: Option<>) -> Option<usize>;
//     /// 在dcache中快速查找目录项
//     /// - `search_path`: 搜索路径
//     /// - `stop_path`: 停止路径
//     /// - 返回值: 找到的`inode`及其`路径` 或 [`None`]
//     fn quick_lookup<'a> (
//         &self,
//         search_path: &'a Path,
//         stop_path: &'a Path,
//     ) -> Option<(Arc<MountFSInode>, &'a Path)>;
// }

impl DCache {
    pub fn new() -> Self {
        DCache {
            table: RwLock::new(HashSet::new()),
            lru_list: RwLock::new(LinkedList::new()),
            max_size: 0,
        }
    }

    pub fn new_with_max_size(size: usize) -> Self {
        DCache {
            table: RwLock::new(HashSet::new()),
            lru_list: RwLock::new(LinkedList::new()),
            max_size: size,
        }
    }

    pub fn put(&self, src: &Arc<MountFSInode>) {
        self.lru_list.write().push_back(src.clone());
        self.table
            .write()
            .insert(Key::Inner(Resource(Arc::downgrade(src))));
        if self.max_size != 0 && self.table.read().len() >= self.max_size {
            self.clean();
        }
    }

    pub fn clean(&self) -> usize {
        return self
            .lru_list
            .write()
            .extract_if(|elem| Arc::weak_count(&elem.inner_inode) <= 1)
            .count();
    }

    pub fn quick_lookup<'b>(
        &self,
        search_path: &'b Path,
        stop_path: &'b Path,
    ) -> Option<(Arc<MountFSInode>, &'b Path)> {
        if let Some(Key::Inner(src)) = self
            .table
            .read()
            .get(&Key::Cmp(Arc::new(String::from(search_path.as_os_str()))))
        {
            if src.0.weak_count() > 1 {
                return Some((src.0.upgrade().unwrap().overlaid_inode(), search_path));
            }
        }
        if let Some(parent) = search_path.parent() {
            if parent == stop_path {
                return None;
            }
            return self.quick_lookup(parent, stop_path);
        }
        return None;
    }
}

impl Default for DCache {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
struct Resource(Weak<MountFSInode>);

impl Keyable for Resource {
    fn key(&self) -> Arc<String> {
        if let Some(src) = self.0.upgrade() {
            return Arc::new(src.abs_path().unwrap().into_os_string());
        }
        return Arc::new(String::new());
    }
}
