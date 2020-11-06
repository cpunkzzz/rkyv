#[cfg(test)]
mod tests {
    use archive::{
        Archive,
        ArchiveBuffer,
        Archived,
        ArchiveRef,
        Write,
    };

    #[repr(align(16))]
    struct Aligned<T>(T);

    impl<T: AsRef<[U]>, U> AsRef<[U]> for Aligned<T> {
        fn as_ref(&self) -> &[U] {
            self.0.as_ref()
        }
    }

    impl<T: AsMut<[U]>, U> AsMut<[U]> for Aligned<T> {
        fn as_mut(&mut self) -> &mut [U] {
            self.0.as_mut()
        }
    }

    const BUFFER_SIZE: usize = 256;

    fn test_archive<T: Archive<Archived = U>, U: PartialEq<T>>(value: &T) {
        let mut writer = ArchiveBuffer::new(Aligned([0u8; BUFFER_SIZE]));
        let pos = writer.archive(value).expect("failed to archive value");
        let buf = writer.into_inner();
        let archived_value = unsafe { &*buf.as_ref().as_ptr().offset(pos as isize).cast::<U>() };
        assert!(archived_value == value);
    }

    fn test_archive_ref<T: ArchiveRef<Archived = U> + ?Sized, U: PartialEq<T> + ?Sized>(value: &T) {
        let mut writer = ArchiveBuffer::new(Aligned([0u8; BUFFER_SIZE]));
        let pos = writer.archive_ref(value).expect("failed to archive ref");
        let buf = writer.into_inner();
        let archived_ref = unsafe { &*buf.as_ref().as_ptr().offset(pos as isize).cast::<T::Reference>() };
        assert!(&**archived_ref == value);
    }

    fn test_archive_container<T: Archive<Archived = U> + core::ops::Deref<Target = TV>, TV: ?Sized, U: core::ops::Deref<Target = TU>, TU: PartialEq<TV> + ?Sized>(value: &T) {
        let mut writer = ArchiveBuffer::new(Aligned([0u8; BUFFER_SIZE]));
        let pos = writer.archive(value).expect("failed to archive ref");
        let buf = writer.into_inner();
        let archived_ref = unsafe { &*buf.as_ref().as_ptr().offset(pos as isize).cast::<U>() };
        assert!(&**archived_ref == &**value);
    }

    #[test]
    fn archive_primitives() {
        test_archive(&());
        test_archive(&true);
        test_archive(&false);
        test_archive(&1234567f32);
        test_archive(&12345678901234f64);
        test_archive(&123i8);
        test_archive(&123456i32);
        test_archive(&1234567890i128);
        test_archive(&123u8);
        test_archive(&123456u32);
        test_archive(&1234567890u128);
        test_archive(&(24, true, 16f32));
        test_archive(&[1, 2, 3, 4, 5, 6]);

        test_archive(&Option::<()>::None);
        test_archive(&Some(42));
    }

    #[test]
    fn archive_refs() {
        test_archive_ref::<[i32; 4], _>(&[1, 2, 3, 4]);
        test_archive_ref::<str, _>("hello world");
        test_archive_ref::<[i32], _>([1, 2, 3, 4].as_ref());
    }

    #[test]
    fn archive_containers() {
        test_archive_container(&Box::new(42));
        test_archive_container(&"hello world".to_string().into_boxed_str());
        test_archive_container(&vec![1, 2, 3, 4].into_boxed_slice());
        test_archive_container(&"hello world".to_string());
        test_archive_container(&vec![1, 2, 3, 4]);
    }

    #[test]
    fn archive_composition() {
        test_archive(&Some(Box::new(42)));
        test_archive(&Some("hello world".to_string().into_boxed_str()));
        test_archive(&Some(vec![1, 2, 3, 4].into_boxed_slice()));
        test_archive(&Some("hello world".to_string()));
        test_archive(&Some(vec![1, 2, 3, 4]));
        test_archive(&Some(Box::new(vec![1, 2, 3, 4])));
    }

    #[test]
    fn archive_hash_map() {
        use std::collections::HashMap;

        test_archive(&HashMap::<i32, i32>::new());

        let mut hash_map = HashMap::new();
        hash_map.insert(1, 2);
        hash_map.insert(3, 4);
        hash_map.insert(5, 6);
        hash_map.insert(7, 8);

        test_archive(&hash_map);

        let mut hash_map = HashMap::new();
        hash_map.insert("hello".to_string(), "world".to_string());
        hash_map.insert("foo".to_string(), "bar".to_string());
        hash_map.insert("baz".to_string(), "bat".to_string());

        let mut writer = ArchiveBuffer::new(Aligned([0u8; BUFFER_SIZE]));
        let pos = writer.archive(&hash_map).expect("failed to archive value");
        let buf = writer.into_inner();
        let archived_value = unsafe { &*buf.as_ref().as_ptr().offset(pos as isize).cast::<Archived<HashMap<String, String>>>() };

        assert!(archived_value.len() == hash_map.len());

        for (key, value) in hash_map.iter() {
            assert!(archived_value.contains_key(key.as_str()));
            assert!(archived_value[key.as_str()].eq(value));
        }

        for (key, value) in archived_value.iter() {
            assert!(hash_map.contains_key(key.as_str()));
            assert!(hash_map[key.as_str()].eq(value));
        }
    }

    #[derive(Archive)]
    struct TestSimple {
        a: (),
        b: i32,
        c: String,
        d: Option<i32>,
    }

    impl PartialEq<TestSimple> for Archived<TestSimple> {
        fn eq(&self, other: &TestSimple) -> bool {
            self.a == other.a && self.b == other.b && self.c == other.c && self.d == other.d
        }
    }

    impl PartialEq<Archived<TestSimple>> for TestSimple {
        fn eq(&self, other: &Archived<TestSimple>) -> bool {
            other.eq(self)
        }
    }

    #[test]
    fn archive_simple_struct() {
        test_archive(&TestSimple {
            a: (),
            b: 42,
            c: "hello world".to_string(),
            d: Some(42),
        });
        test_archive(&vec![
            TestSimple {
                a: (),
                b: 42,
                c: "hello world".to_string(),
                d: Some(42),
            },
            TestSimple {
                a: (),
                b: 42,
                c: "hello world".to_string(),
                d: Some(42),
            }
        ])
    }
}