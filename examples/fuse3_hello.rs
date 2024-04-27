use std::env;
use std::ffi::{OsStr, OsString};
use std::future::Future;
use std::iter::Skip;
use std::num::NonZeroU32;
use std::time::{Duration, SystemTime};
use std::vec::IntoIter;

use bytes::Bytes;
use fuse3::raw::prelude::*;
use fuse3::{Inode, MountOptions, Result};
use futures_util::stream;
use futures_util::stream::Iter;
use tracing::{info, instrument, Level};


const CONTENT: &str = "hello world\n";

const PARENT_INODE: u64 = 1;
const FILE_INODE: u64 = 2;
const FILE_NAME: &str = "hello-world.txt";
const PARENT_MODE: u16 = 0o755;
const FILE_MODE: u16 = 0o644;
const TTL: Duration = Duration::from_secs(1);
const STATFS: ReplyStatFs = ReplyStatFs {
    blocks: 1,
    bfree: 0,
    bavail: 0,
    files: 1,
    ffree: 0,
    bsize: 4096,
    namelen: u32::MAX,
    frsize: 0,
};

struct HelloWorld;

impl Filesystem for HelloWorld {
    async fn init(&self, _req: Request) -> Result<ReplyInit> {
        Ok(ReplyInit {
            max_write: NonZeroU32::new(1024 * 1024).unwrap(),
        })
    }

    async fn destroy(&self, _req: Request) {}

    #[instrument(skip(self))]
    async fn lookup(&self, _req: Request, parent: u64, name: &OsStr) -> Result<ReplyEntry> {
        info!("");

        if parent != PARENT_INODE {
            return Err(libc::ENOENT.into());
        }

        if name != OsStr::new(FILE_NAME) {
            return Err(libc::ENOENT.into());
        }

        Ok(ReplyEntry {
            ttl: TTL,
            attr: FileAttr {
                ino: FILE_INODE,
                size: CONTENT.len() as u64,
                blocks: 0,
                atime: SystemTime::now().into(),
                mtime: SystemTime::now().into(),
                ctime: SystemTime::now().into(),
                kind: FileType::RegularFile,
                perm: FILE_MODE,
                nlink: 0,
                uid: 0,
                gid: 0,
                rdev: 0,
                blksize: 0,
            },
            generation: 0,
        })
    }

    #[instrument(skip(self))]
    async fn getattr(
        &self,
        _req: Request,
        inode: u64,
        _fh: Option<u64>,
        _flags: u32,
    ) -> Result<ReplyAttr> {
        info!("");

        if inode == PARENT_INODE {
            Ok(ReplyAttr {
                ttl: TTL,
                attr: FileAttr {
                    ino: PARENT_INODE,
                    size: 0,
                    blocks: 0,
                    atime: SystemTime::now().into(),
                    mtime: SystemTime::now().into(),
                    ctime: SystemTime::now().into(),
                    kind: FileType::Directory,
                    perm: PARENT_MODE,
                    nlink: 0,
                    uid: 0,
                    gid: 0,
                    rdev: 0,
                    blksize: 0,
                },
            })
        } else if inode == FILE_INODE {
            Ok(ReplyAttr {
                ttl: TTL,
                attr: FileAttr {
                    ino: FILE_INODE,
                    size: CONTENT.len() as _,
                    blocks: 0,
                    atime: SystemTime::now().into(),
                    mtime: SystemTime::now().into(),
                    ctime: SystemTime::now().into(),
                    kind: FileType::RegularFile,
                    perm: FILE_MODE,
                    nlink: 0,
                    uid: 0,
                    gid: 0,
                    rdev: 0,
                    blksize: 0,
                },
            })
        } else {
            Err(libc::ENOENT.into())
        }
    }

    #[instrument(skip(self))]
    async fn open(&self, _req: Request, inode: u64, flags: u32) -> Result<ReplyOpen> {
        info!("");

        if inode != PARENT_INODE && inode != FILE_INODE {
            return Err(libc::ENOENT.into());
        }

        Ok(ReplyOpen { fh: 1, flags })
    }

    #[instrument(skip(self))]
    async fn read(
        &self,
        _req: Request,
        inode: u64,
        _fh: u64,
        offset: u64,
        size: u32,
    ) -> Result<ReplyData> {
        info!("");

        if inode != FILE_INODE {
            return Err(libc::ENOENT.into());
        }

        if offset as usize >= CONTENT.len() {
            Ok(ReplyData { data: Bytes::new() })
        } else {
            let mut data = &CONTENT.as_bytes()[offset as usize..];

            if data.len() > size as usize {
                data = &data[..size as usize];
            }

            Ok(ReplyData {
                data: Bytes::copy_from_slice(data),
            })
        }
    }

    type DirEntryStream<'a> = Iter<Skip<IntoIter<Result<DirectoryEntry>>>> where Self: 'a;

    #[instrument(skip(self))]
    async fn readdir(
        &self,
        _req: Request,
        inode: u64,
        _fh: u64,
        offset: i64,
    ) -> Result<ReplyDirectory<Self::DirEntryStream<'_>>> {
        info!("");

        if inode == FILE_INODE {
            return Err(libc::ENOTDIR.into());
        }

        if inode != PARENT_INODE {
            return Err(libc::ENOENT.into());
        }

        let entries = vec![
            Ok(DirectoryEntry {
                inode: PARENT_INODE,
                kind: FileType::Directory,
                name: OsString::from("."),
                offset: 1,
            }),
            Ok(DirectoryEntry {
                inode: PARENT_INODE,
                kind: FileType::Directory,
                name: OsString::from(".."),
                offset: 2,
            }),
            Ok(DirectoryEntry {
                inode: FILE_INODE,
                kind: FileType::RegularFile,
                name: OsString::from(FILE_NAME),
                offset: 3,
            }),
        ];

        Ok(ReplyDirectory {
            entries: stream::iter(entries.into_iter().skip(offset as usize)),
        })
    }

    #[instrument(skip(self))]
    async fn access(&self, _req: Request, inode: u64, _mask: u32) -> Result<()> {
        info!("");

        if inode != PARENT_INODE && inode != FILE_INODE {
            return Err(libc::ENOENT.into());
        }

        Ok(())
    }

    type DirEntryPlusStream<'a> = Iter<Skip<IntoIter<Result<DirectoryEntryPlus>>>> where Self: 'a;

    #[instrument(skip(self))]
    async fn readdirplus(
        &self,
        _req: Request,
        parent: u64,
        _fh: u64,
        offset: u64,
        _lock_owner: u64,
    ) -> Result<ReplyDirectoryPlus<Self::DirEntryPlusStream<'_>>> {
        info!("");

        if parent == FILE_INODE {
            return Err(libc::ENOTDIR.into());
        }

        if parent != PARENT_INODE {
            return Err(libc::ENOENT.into());
        }

        let entries = vec![
            Ok(DirectoryEntryPlus {
                inode: PARENT_INODE,
                generation: 0,
                kind: FileType::Directory,
                name: OsString::from("."),
                offset: 1,
                attr: FileAttr {
                    ino: PARENT_INODE,
                    size: 0,
                    blocks: 0,
                    atime: SystemTime::now().into(),
                    mtime: SystemTime::now().into(),
                    ctime: SystemTime::now().into(),
                    kind: FileType::Directory,
                    perm: PARENT_MODE,
                    nlink: 0,
                    uid: 0,
                    gid: 0,
                    rdev: 0,
                    blksize: 0,
                },
                entry_ttl: TTL,
                attr_ttl: TTL,
            }),
            Ok(DirectoryEntryPlus {
                inode: PARENT_INODE,
                generation: 0,
                kind: FileType::Directory,
                name: OsString::from(".."),
                offset: 2,
                attr: FileAttr {
                    ino: PARENT_INODE,
                    size: 0,
                    blocks: 0,
                    atime: SystemTime::now().into(),
                    mtime: SystemTime::now().into(),
                    ctime: SystemTime::now().into(),
                    kind: FileType::Directory,
                    perm: PARENT_MODE,
                    nlink: 0,
                    uid: 0,
                    gid: 0,
                    rdev: 0,
                    blksize: 0,
                },
                entry_ttl: TTL,
                attr_ttl: TTL,
            }),
            Ok(DirectoryEntryPlus {
                inode: FILE_INODE,
                generation: 0,
                kind: FileType::Directory,
                name: OsString::from(FILE_NAME),
                offset: 3,
                attr: FileAttr {
                    ino: FILE_INODE,
                    size: CONTENT.len() as _,
                    blocks: 0,
                    atime: SystemTime::now().into(),
                    mtime: SystemTime::now().into(),
                    ctime: SystemTime::now().into(),
                    kind: FileType::RegularFile,
                    perm: FILE_MODE,
                    nlink: 0,
                    uid: 0,
                    gid: 0,
                    rdev: 0,
                    blksize: 0,
                },
                entry_ttl: TTL,
                attr_ttl: TTL,
            }),
        ];

        Ok(ReplyDirectoryPlus {
            entries: stream::iter(entries.into_iter().skip(offset as usize)),
        })
    }

    async fn statfs(&self, _req: Request, _inode: u64) -> Result<ReplyStatFs> {
        Ok(STATFS)
    }

    #[instrument(skip(self))]
    async fn flush(&self, req: Request, inode: Inode, fh: u64, lock_owner: u64) -> Result<()> {
        info!("");

        Ok(())
    }

    #[instrument(skip(self, data))]
    async fn write(
        &self,
        req: Request,
        inode: Inode,
        fh: u64,
        offset: u64,
        data: &[u8],
        write_flags: u32,
        flags: u32,
    ) -> Result<ReplyWrite> {
        // info!("");

        Ok(ReplyWrite {
            written: data.len() as u32,
        })
    }

    #[instrument(skip(self))]
    async fn release(
        &self,
        req: Request,
        inode: Inode,
        fh: u64,
        flags: u32,
        lock_owner: u64,
        flush: bool,
    ) -> Result<()> {
        info!("");

        Ok(())
    }

    #[instrument(skip(self))]
    async fn setattr(
        &self,
        req: Request,
        inode: Inode,
        fh: Option<u64>,
        set_attr: SetAttr,
    ) -> Result<ReplyAttr> {
        info!("");

        Ok(ReplyAttr {
            ttl: TTL,
            attr: FileAttr {
                ino: inode,
                size: 0,
                blocks: 0,
                atime: SystemTime::now().into(),
                mtime: SystemTime::now().into(),
                ctime: SystemTime::now().into(),
                kind: FileType::RegularFile,
                perm: FILE_MODE,
                nlink: 0,
                uid: 0,
                gid: 0,
                rdev: 0,
                blksize: 0,
            },
        })
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() {
    log_init();

    let args = env::args_os().skip(1).take(1).collect::<Vec<_>>();

    let mount_path = args.first();

    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };

    let mut mount_options = MountOptions::default();
    mount_options.uid(uid).gid(gid).read_only(false);

    let mount_path = mount_path.expect("no mount point specified");
    Session::new(mount_options)
        .mount_with_unprivileged(HelloWorld {}, mount_path)
        .await
        .unwrap()
        .await
        .unwrap()
}

fn log_init() {
    let subscriber = tracing_subscriber::fmt()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber).unwrap();
}
