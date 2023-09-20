pub trait SignalRContractSerializer {
    const ACTION_NAME: &'static str;
    fn serialize(&self) -> Vec<Vec<u8>>;
    fn deserialize<'s>(src: &'s [Vec<u8>]) -> Self;
}
