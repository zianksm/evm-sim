pub trait StorageProvider {
    fn load_ext(&self, key: &[u8]) -> Option<Vec<u8>>;
}