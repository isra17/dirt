/// DataType encapsulate different kind of data that can be used as
/// input or output for an emulated function.
pub trait DataType {
    /// Pushable value returns a value that can be pushed on a machine stack.
    /// for scalar type it can return its raw value, but for complexe type and
    /// pointer, it probably need to write the data in |mem| and then return
    /// a pointer to this data.
    fn pushable_value(&self, data: DataStore) -> u64;
}

struct StringData {
    value: String,
}

impl DataType for StringData {
    fn pushable_value(&self, vmstate: EmuData) -> u64 {
        let ptr = emudata.write_str(self.value);
        return ptr;
    }
}
