pub trait SignalRContractSerializer {
    const ACTION_NAME: &'static str;
    type Item;
    fn serialize(&self) -> Vec<Vec<u8>>;
    fn deserialize<'s>(src: impl Iterator<Item = &'s [u8]>) -> Result<Self::Item, String>;
}
