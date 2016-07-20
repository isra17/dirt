use emu::Error;
use emu::vmstate::DataWriter;
use std::rc::Rc;

/// DataType encapsulate different kind of data that can be used as
/// input or output for an emulated function.
pub trait DataType: ::std::fmt::Debug {
    /// Pushable value returns a value that can be pushed on a machine stack.
    /// for scalar type it can return its raw value, but for complexe type and
    /// pointer, it probably need to write the data in |mem| and then return
    /// a pointer to this data.
    fn pushable_value(&self,
                      data_writer: &mut DataWriter)
                      -> Result<u64, Error>;
    fn write_value(&self, data_writer: &mut DataWriter) -> Result<(), Error>;
}

#[derive(Debug)]
pub struct StringData {
    value: String,
}

impl DataType for StringData {
    fn pushable_value(&self,
                      data_writer: &mut DataWriter)
                      -> Result<u64, Error> {
        return data_writer.write_str(self.value.as_str());
    }

    fn write_value(&self, data_writer: &mut DataWriter) -> Result<(), Error> {
        try!(self.pushable_value(data_writer));
        return Ok(());
    }
}

impl StringData {
    pub fn new(value: &str) -> StringData {
        return StringData { value: String::from(value) };
    }
}

#[derive(Debug)]
pub struct BufData {
    size: u64,
    data: Option<Vec<u8>>,
}

impl DataType for BufData {
    fn pushable_value(&self,
                      data_writer: &mut DataWriter)
                      -> Result<u64, Error> {
        let mut data = self.data
            .clone()
            .unwrap_or_else(|| Vec::with_capacity(self.size as usize));
        data.resize(self.size as usize, 0);
        return data_writer.write_data(&data);
    }

    fn write_value(&self, data_writer: &mut DataWriter) -> Result<(), Error> {
        try!(self.pushable_value(data_writer));
        return Ok(());
    }
}

impl BufData {
    pub fn new(size: u64, data: Option<Vec<u8>>) -> BufData {
        return BufData {
            size: size,
            data: data,
        };
    }
}

#[derive(Debug)]
pub struct IntegerData(pub u64);

impl DataType for IntegerData {
    fn pushable_value(&self, _: &mut DataWriter) -> Result<u64, Error> {
        return Ok(self.0);
    }

    fn write_value(&self, data_writer: &mut DataWriter) -> Result<(), Error> {
        try!(data_writer.write_usize(self.0));
        return Ok(());
    }
}

#[derive(Debug)]
pub struct CompositeData {
    fields: Vec<Rc<DataType>>,
}

impl DataType for CompositeData {
    fn pushable_value(&self,
                      data_writer: &mut DataWriter)
                      -> Result<u64, Error> {
        let ptr = data_writer.current_ptr();
        for field in &self.fields {
            try!(field.write_value(data_writer));
        }

        return Ok(ptr);
    }

    fn write_value(&self, data_writer: &mut DataWriter) -> Result<(), Error> {
        try!(self.pushable_value(data_writer));
        return Ok(());
    }
}

impl CompositeData {
    pub fn new(fields: Vec<Rc<DataType>>) -> CompositeData {
        return CompositeData { fields: fields };
    }
}

#[derive(Debug)]
pub struct ThisOffsetData(pub u64);

impl DataType for ThisOffsetData {
    fn pushable_value(&self,
                      data_writer: &mut DataWriter)
                      -> Result<u64, Error> {
        return Ok(data_writer.current_ptr() + self.0);
    }

    fn write_value(&self, data_writer: &mut DataWriter) -> Result<(), Error> {
        let value = self.pushable_value(data_writer).unwrap();
        try!(data_writer.write_usize(value));
        return Ok(());
    }
}

#[derive(Debug)]
pub struct ByteData(pub u8);

impl DataType for ByteData {
    fn pushable_value(&self, _: &mut DataWriter) -> Result<u64, Error> {
        return Ok(self.0 as u64);
    }

    fn write_value(&self, data_writer: &mut DataWriter) -> Result<(), Error> {
        try!(data_writer.write_data(&[self.0]));
        return Ok(());
    }
}
