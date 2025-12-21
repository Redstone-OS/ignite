//! RedstoneFS - Implementação básica do sistema de arquivos
//!
//! Este módulo contém as definições e implementações para leitura do
//! RedstoneFS.

use core::marker::PhantomData;

use crate::hardware::io::{Io, ReadOnly};

pub const BLOCK_SIZE: u64 = 4096;
pub const RECORD_SIZE: u64 = 4096;

pub trait Disk {
    fn read_at(&mut self, block: u64, buffer: &mut [u8]) -> syscall::Result<usize>;
    fn write_at(&mut self, block: u64, buffer: &[u8]) -> syscall::Result<usize>;
    fn size(&mut self) -> syscall::Result<u64>;
}

pub struct Header {
    uuid: [u8; 16],
    size: u64,
}

impl Header {
    pub fn uuid(&self) -> &[u8] {
        &self.uuid
    }
    pub fn size(&self) -> u64 {
        self.size
    }
}

pub struct FileSystem<D> {
    pub disk:   D,
    pub block:  u64,
    pub header: Header,
}

impl<D: Disk> FileSystem<D> {
    pub fn open(
        disk: D,
        _password: Option<&[u8]>,
        block_opt: Option<u64>,
        _readonly: bool,
    ) -> syscall::Result<Self> {
        Ok(Self {
            disk,
            block: block_opt.unwrap_or(0),
            header: Header {
                uuid: [0; 16],
                size: 0,
            },
        })
    }

    pub fn tx<F, T>(&mut self, f: F) -> syscall::Result<T>
    where
        F: FnOnce(&mut Transaction<D>) -> syscall::Result<T>,
    {
        let mut tx = Transaction { fs: self };
        f(&mut tx)
    }
}

pub struct Transaction<'a, D: 'a> {
    fs: &'a mut FileSystem<D>,
}

impl<'a, D: Disk> Transaction<'a, D> {
    pub fn find_node(&mut self, _ptr: TreePtr, _name: &str) -> syscall::Result<Node> {
        // Stub
        Err(syscall::Error::new(syscall::ENOENT))
    }

    pub fn read_node_inner(
        &mut self,
        _node: &Node,
        _offset: u64,
        _buf: &mut [u8],
    ) -> syscall::Result<usize> {
        // Stub
        Ok(0)
    }
}

pub struct TreePtr {
    // Stub
}

impl TreePtr {
    pub fn root() -> Self {
        Self {}
    }
    pub fn ptr(&self) -> Self {
        Self {}
    }
}

pub struct Node {
    // Stub
}

impl Node {
    pub fn ptr(&self) -> TreePtr {
        TreePtr {}
    }

    pub fn data(&self) -> NodeData {
        NodeData {}
    }
}

pub struct NodeData {}

impl NodeData {
    pub fn size(&self) -> u64 {
        0
    }
}
