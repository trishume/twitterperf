use std::ops::Index;
use std::sync::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::ptr::{NonNull, self};
use std::io;

pub struct SharedPool<T> {
    len: AtomicUsize,
    lock: Mutex<()>,
    buf: NonNull<T>,
}

// https://vgel.me/posts/mmap-arena-alloc/
impl<T> SharedPool<T> {
    /// It doesn't matter that much how large this is but let's go for 34GB
    const MAP_SIZE: usize = 1 << 35;

    pub fn new() -> io::Result<Self> {
        // TODO use hugepages only on linux
        let map = unsafe {
            libc::mmap(
                ptr::null_mut(),
                Self::MAP_SIZE,
                libc::PROT_READ | libc::PROT_WRITE,
                // MAP_PRIVATE:   this is not shared memory
                // MAP_ANONYMOUS: this is RAM, not a file-backed mmap
                // MAP_NORESERVE: don't reserve swap
                // MAP_HUGETLB:   use huge pages for better performance
                //                (make sure huge pages are enabled or this will SIGBUS:
                //                 # sysctl -w vm.nr_hugepages=2048)
                libc::MAP_PRIVATE | libc::MAP_ANONYMOUS | libc::MAP_NORESERVE, // | libc::MAP_HUGETLB,
                -1,
                0,
            )
        };

        let buf = if map == libc::MAP_FAILED || map.is_null() {
            return Err(io::Error::last_os_error());
        } else {
            NonNull::new(map as *mut T).unwrap()
        };

        Ok(Self {
            buf,
            lock: Mutex::new(()),
            len: AtomicUsize::new(0),
        })
    }

    #[inline]
    pub fn push(&self, value: T) -> usize {
        // TODO either be clever about queueing these up or
        // split this type into a reader and a writer to avoid the lock
        let _guard = self.lock.lock();
        let i = self.len.load(Ordering::SeqCst);
        unsafe {
            let end = self.buf.as_ptr().add(i);
            ptr::write(end, value);
        }
        self.len.fetch_add(1, Ordering::SeqCst);
        i
    }
}

impl<T> Index<usize> for SharedPool<T> {
    type Output = T;

    #[inline]
    fn index(&self, i: usize) -> &T {
        let len = self.len.load(Ordering::SeqCst);
        if i >= len { panic!("index out of bounds {i} for length {len}")}
        unsafe {
            let item = self.buf.as_ptr().add(i);
            & *item
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic_pool() {
        let pool = SharedPool::new().unwrap();
        pool.push(5);
        pool.push(6);
        assert_eq!(pool[0], 5);
        assert_eq!(pool[1], 6);
    }
}
