#![allow(warnings)]

use bebop::FixedSized as _;
use core::convert::TryInto as _;
use std::io::Write as _;

pub const PROTOCOL_VERSION: u32 = 1;

pub const GRID_OPCODE: u32 = 1;

pub const PIXEL_OPCODE: u32 = 2;

pub const DELTA_GRID_OPCODE: u32 = 3;

#[derive(Clone, Debug, PartialEq)]
pub struct BebopData<'raw> {
    pub protocol_version: u32,
    pub opcode: u32,
    pub encoded_data: ::bebop::SliceWrapper<'raw, u8>,
}

impl<'raw> ::bebop::SubRecord<'raw> for BebopData<'raw> {
    const MIN_SERIALIZED_SIZE: usize = <u32>::MIN_SERIALIZED_SIZE
        + <u32>::MIN_SERIALIZED_SIZE
        + <::bebop::SliceWrapper<'raw, u8>>::MIN_SERIALIZED_SIZE;

    #[inline]
    fn serialized_size(&self) -> usize {
        self.protocol_version.serialized_size()
            + self.opcode.serialized_size()
            + self.encoded_data.serialized_size()
    }

    ::bebop::define_serialize_chained!(Self => |zelf, dest| {
        Ok(
            zelf.protocol_version._serialize_chained(dest)? +
            zelf.opcode._serialize_chained(dest)? +
            zelf.encoded_data._serialize_chained(dest)?
        )
    });

    fn _deserialize_chained(raw: &'raw [u8]) -> ::bebop::DeResult<(usize, Self)> {
        let mut i = 0;
        if raw.len() - i < Self::MIN_SERIALIZED_SIZE {
            let missing = Self::MIN_SERIALIZED_SIZE - (raw.len() - i);
            return Err(::bebop::DeserializeError::MoreDataExpected(missing));
        }

        let (read, v0) = ::bebop::SubRecord::_deserialize_chained(&raw[i..])?;
        i += read;
        let (read, v1) = ::bebop::SubRecord::_deserialize_chained(&raw[i..])?;
        i += read;
        let (read, v2) = ::bebop::SubRecord::_deserialize_chained(&raw[i..])?;
        i += read;

        Ok((
            i,
            Self {
                protocol_version: v0,
                opcode: v1,
                encoded_data: v2,
            },
        ))
    }
}

impl<'raw> ::bebop::Record<'raw> for BebopData<'raw> {}

#[derive(Clone, Debug, PartialEq)]
pub struct Grid<'raw> {
    pub rows: ::std::vec::Vec<Row<'raw>>,
}

impl<'raw> ::bebop::SubRecord<'raw> for Grid<'raw> {
    const MIN_SERIALIZED_SIZE: usize = <::std::vec::Vec<Row<'raw>>>::MIN_SERIALIZED_SIZE;

    #[inline]
    fn serialized_size(&self) -> usize {
        self.rows.serialized_size()
    }

    ::bebop::define_serialize_chained!(Self => |zelf, dest| {
        Ok(
            zelf.rows._serialize_chained(dest)?
        )
    });

    fn _deserialize_chained(raw: &'raw [u8]) -> ::bebop::DeResult<(usize, Self)> {
        let mut i = 0;
        if raw.len() - i < Self::MIN_SERIALIZED_SIZE {
            let missing = Self::MIN_SERIALIZED_SIZE - (raw.len() - i);
            return Err(::bebop::DeserializeError::MoreDataExpected(missing));
        }

        let (read, v0) = ::bebop::SubRecord::_deserialize_chained(&raw[i..])?;
        i += read;

        Ok((i, Self { rows: v0 }))
    }
}

impl<'raw> ::bebop::Record<'raw> for Grid<'raw> {}

#[derive(Clone, Debug, PartialEq)]
pub struct Row<'raw> {
    pub pixels: ::bebop::SliceWrapper<'raw, Color>,
}

impl<'raw> ::bebop::SubRecord<'raw> for Row<'raw> {
    const MIN_SERIALIZED_SIZE: usize = <::bebop::SliceWrapper<'raw, Color>>::MIN_SERIALIZED_SIZE;

    #[inline]
    fn serialized_size(&self) -> usize {
        self.pixels.serialized_size()
    }

    ::bebop::define_serialize_chained!(Self => |zelf, dest| {
        Ok(
            zelf.pixels._serialize_chained(dest)?
        )
    });

    fn _deserialize_chained(raw: &'raw [u8]) -> ::bebop::DeResult<(usize, Self)> {
        let mut i = 0;
        if raw.len() - i < Self::MIN_SERIALIZED_SIZE {
            let missing = Self::MIN_SERIALIZED_SIZE - (raw.len() - i);
            return Err(::bebop::DeserializeError::MoreDataExpected(missing));
        }

        let (read, v0) = ::bebop::SubRecord::_deserialize_chained(&raw[i..])?;
        i += read;

        Ok((i, Self { pixels: v0 }))
    }
}

impl<'raw> ::bebop::Record<'raw> for Row<'raw> {}

#[derive(Clone, Debug, PartialEq, Copy)]
#[repr(packed)]
pub struct Pixel {
    pub x: u32,
    pub y: u32,
    pub color: Color,
}

impl ::bebop::FixedSized for Pixel {}

impl<'raw> ::bebop::SubRecord<'raw> for Pixel {
    const MIN_SERIALIZED_SIZE: usize = Self::SERIALIZED_SIZE;
    const EXACT_SERIALIZED_SIZE: Option<usize> = Some(Self::SERIALIZED_SIZE);

    #[inline]
    fn serialized_size(&self) -> usize {
        Self::SERIALIZED_SIZE
    }

    ::bebop::define_serialize_chained!(*Self => |zelf, dest| {
        Ok(
            ::bebop::packed_read!(zelf.x)._serialize_chained(dest)? +
            ::bebop::packed_read!(zelf.y)._serialize_chained(dest)? +
            ::bebop::packed_read!(zelf.color)._serialize_chained(dest)?
        )
    });

    fn _deserialize_chained(raw: &'raw [u8]) -> ::bebop::DeResult<(usize, Self)> {
        let mut i = 0;
        if raw.len() - i < Self::MIN_SERIALIZED_SIZE {
            let missing = Self::MIN_SERIALIZED_SIZE - (raw.len() - i);
            return Err(::bebop::DeserializeError::MoreDataExpected(missing));
        }

        let (read, v0) = ::bebop::SubRecord::_deserialize_chained(&raw[i..])?;
        i += read;
        let (read, v1) = ::bebop::SubRecord::_deserialize_chained(&raw[i..])?;
        i += read;
        let (read, v2) = ::bebop::SubRecord::_deserialize_chained(&raw[i..])?;
        i += read;

        Ok((
            i,
            Self {
                x: v0,
                y: v1,
                color: v2,
            },
        ))
    }
}

impl<'raw> ::bebop::Record<'raw> for Pixel {}

#[derive(Clone, Debug, PartialEq, Copy)]
#[repr(packed)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl ::bebop::FixedSized for Color {}

impl<'raw> ::bebop::SubRecord<'raw> for Color {
    const MIN_SERIALIZED_SIZE: usize = Self::SERIALIZED_SIZE;
    const EXACT_SERIALIZED_SIZE: Option<usize> = Some(Self::SERIALIZED_SIZE);

    #[inline]
    fn serialized_size(&self) -> usize {
        Self::SERIALIZED_SIZE
    }

    ::bebop::define_serialize_chained!(*Self => |zelf, dest| {
        Ok(
            ::bebop::packed_read!(zelf.red)._serialize_chained(dest)? +
            ::bebop::packed_read!(zelf.green)._serialize_chained(dest)? +
            ::bebop::packed_read!(zelf.blue)._serialize_chained(dest)?
        )
    });

    fn _deserialize_chained(raw: &'raw [u8]) -> ::bebop::DeResult<(usize, Self)> {
        let mut i = 0;
        if raw.len() - i < Self::MIN_SERIALIZED_SIZE {
            let missing = Self::MIN_SERIALIZED_SIZE - (raw.len() - i);
            return Err(::bebop::DeserializeError::MoreDataExpected(missing));
        }

        let (read, v0) = ::bebop::SubRecord::_deserialize_chained(&raw[i..])?;
        i += read;
        let (read, v1) = ::bebop::SubRecord::_deserialize_chained(&raw[i..])?;
        i += read;
        let (read, v2) = ::bebop::SubRecord::_deserialize_chained(&raw[i..])?;
        i += read;

        Ok((
            i,
            Self {
                red: v0,
                green: v1,
                blue: v2,
            },
        ))
    }
}

impl<'raw> ::bebop::Record<'raw> for Color {}

#[derive(Clone, Debug, PartialEq)]
pub struct DeltaGrid<'raw> {
    pub delta: ::bebop::SliceWrapper<'raw, Pixel>,
}

impl<'raw> ::bebop::SubRecord<'raw> for DeltaGrid<'raw> {
    const MIN_SERIALIZED_SIZE: usize = <::bebop::SliceWrapper<'raw, Pixel>>::MIN_SERIALIZED_SIZE;

    #[inline]
    fn serialized_size(&self) -> usize {
        self.delta.serialized_size()
    }

    ::bebop::define_serialize_chained!(Self => |zelf, dest| {
        Ok(
            zelf.delta._serialize_chained(dest)?
        )
    });

    fn _deserialize_chained(raw: &'raw [u8]) -> ::bebop::DeResult<(usize, Self)> {
        let mut i = 0;
        if raw.len() - i < Self::MIN_SERIALIZED_SIZE {
            let missing = Self::MIN_SERIALIZED_SIZE - (raw.len() - i);
            return Err(::bebop::DeserializeError::MoreDataExpected(missing));
        }

        let (read, v0) = ::bebop::SubRecord::_deserialize_chained(&raw[i..])?;
        i += read;

        Ok((i, Self { delta: v0 }))
    }
}

impl<'raw> ::bebop::Record<'raw> for DeltaGrid<'raw> {}

#[cfg(feature = "bebop-owned-all")]
pub mod owned {
    #![allow(warnings)]

    use bebop::FixedSized as _;
    use core::convert::TryInto as _;
    use std::io::Write as _;

    pub use super::PROTOCOL_VERSION;

    pub use super::GRID_OPCODE;

    pub use super::PIXEL_OPCODE;

    pub use super::DELTA_GRID_OPCODE;

    #[derive(Clone, Debug, PartialEq)]
    pub struct BebopData {
        pub protocol_version: u32,
        pub opcode: u32,
        pub encoded_data: ::std::vec::Vec<u8>,
    }

    impl<'raw> ::core::convert::From<super::BebopData<'raw>> for BebopData {
        fn from(value: super::BebopData) -> Self {
            Self {
                protocol_version: value.protocol_version,
                opcode: value.opcode,
                encoded_data: value.encoded_data.iter().map(|value| value).collect(),
            }
        }
    }

    impl<'raw> ::bebop::SubRecord<'raw> for BebopData {
        const MIN_SERIALIZED_SIZE: usize = <u32>::MIN_SERIALIZED_SIZE
            + <u32>::MIN_SERIALIZED_SIZE
            + <::std::vec::Vec<u8>>::MIN_SERIALIZED_SIZE;

        #[inline]
        fn serialized_size(&self) -> usize {
            self.protocol_version.serialized_size()
                + self.opcode.serialized_size()
                + self.encoded_data.serialized_size()
        }

        ::bebop::define_serialize_chained!(Self => |zelf, dest| {
            Ok(
                zelf.protocol_version._serialize_chained(dest)? +
                zelf.opcode._serialize_chained(dest)? +
                zelf.encoded_data._serialize_chained(dest)?
            )
        });

        fn _deserialize_chained(raw: &'raw [u8]) -> ::bebop::DeResult<(usize, Self)> {
            let mut i = 0;
            if raw.len() - i < Self::MIN_SERIALIZED_SIZE {
                let missing = Self::MIN_SERIALIZED_SIZE - (raw.len() - i);
                return Err(::bebop::DeserializeError::MoreDataExpected(missing));
            }

            let (read, v0) = ::bebop::SubRecord::_deserialize_chained(&raw[i..])?;
            i += read;
            let (read, v1) = ::bebop::SubRecord::_deserialize_chained(&raw[i..])?;
            i += read;
            let (read, v2) = ::bebop::SubRecord::_deserialize_chained(&raw[i..])?;
            i += read;

            Ok((
                i,
                Self {
                    protocol_version: v0,
                    opcode: v1,
                    encoded_data: v2,
                },
            ))
        }
    }

    impl<'raw> ::bebop::Record<'raw> for BebopData {}

    #[derive(Clone, Debug, PartialEq)]
    pub struct Grid {
        pub rows: ::std::vec::Vec<Row>,
    }

    impl<'raw> ::core::convert::From<super::Grid<'raw>> for Grid {
        fn from(value: super::Grid) -> Self {
            Self {
                rows: value.rows.into_iter().map(|value| value.into()).collect(),
            }
        }
    }

    impl<'raw> ::bebop::SubRecord<'raw> for Grid {
        const MIN_SERIALIZED_SIZE: usize = <::std::vec::Vec<Row>>::MIN_SERIALIZED_SIZE;

        #[inline]
        fn serialized_size(&self) -> usize {
            self.rows.serialized_size()
        }

        ::bebop::define_serialize_chained!(Self => |zelf, dest| {
            Ok(
                zelf.rows._serialize_chained(dest)?
            )
        });

        fn _deserialize_chained(raw: &'raw [u8]) -> ::bebop::DeResult<(usize, Self)> {
            let mut i = 0;
            if raw.len() - i < Self::MIN_SERIALIZED_SIZE {
                let missing = Self::MIN_SERIALIZED_SIZE - (raw.len() - i);
                return Err(::bebop::DeserializeError::MoreDataExpected(missing));
            }

            let (read, v0) = ::bebop::SubRecord::_deserialize_chained(&raw[i..])?;
            i += read;

            Ok((i, Self { rows: v0 }))
        }
    }

    impl<'raw> ::bebop::Record<'raw> for Grid {}

    #[derive(Clone, Debug, PartialEq)]
    pub struct Row {
        pub pixels: ::std::vec::Vec<Color>,
    }

    impl<'raw> ::core::convert::From<super::Row<'raw>> for Row {
        fn from(value: super::Row) -> Self {
            Self {
                pixels: value.pixels.iter().map(|value| value).collect(),
            }
        }
    }

    impl<'raw> ::bebop::SubRecord<'raw> for Row {
        const MIN_SERIALIZED_SIZE: usize = <::std::vec::Vec<Color>>::MIN_SERIALIZED_SIZE;

        #[inline]
        fn serialized_size(&self) -> usize {
            self.pixels.serialized_size()
        }

        ::bebop::define_serialize_chained!(Self => |zelf, dest| {
            Ok(
                zelf.pixels._serialize_chained(dest)?
            )
        });

        fn _deserialize_chained(raw: &'raw [u8]) -> ::bebop::DeResult<(usize, Self)> {
            let mut i = 0;
            if raw.len() - i < Self::MIN_SERIALIZED_SIZE {
                let missing = Self::MIN_SERIALIZED_SIZE - (raw.len() - i);
                return Err(::bebop::DeserializeError::MoreDataExpected(missing));
            }

            let (read, v0) = ::bebop::SubRecord::_deserialize_chained(&raw[i..])?;
            i += read;

            Ok((i, Self { pixels: v0 }))
        }
    }

    impl<'raw> ::bebop::Record<'raw> for Row {}

    pub use super::Pixel;

    pub use super::Color;

    #[derive(Clone, Debug, PartialEq)]
    pub struct DeltaGrid {
        pub delta: ::std::vec::Vec<Pixel>,
    }

    impl<'raw> ::core::convert::From<super::DeltaGrid<'raw>> for DeltaGrid {
        fn from(value: super::DeltaGrid) -> Self {
            Self {
                delta: value.delta.iter().map(|value| value).collect(),
            }
        }
    }

    impl<'raw> ::bebop::SubRecord<'raw> for DeltaGrid {
        const MIN_SERIALIZED_SIZE: usize = <::std::vec::Vec<Pixel>>::MIN_SERIALIZED_SIZE;

        #[inline]
        fn serialized_size(&self) -> usize {
            self.delta.serialized_size()
        }

        ::bebop::define_serialize_chained!(Self => |zelf, dest| {
            Ok(
                zelf.delta._serialize_chained(dest)?
            )
        });

        fn _deserialize_chained(raw: &'raw [u8]) -> ::bebop::DeResult<(usize, Self)> {
            let mut i = 0;
            if raw.len() - i < Self::MIN_SERIALIZED_SIZE {
                let missing = Self::MIN_SERIALIZED_SIZE - (raw.len() - i);
                return Err(::bebop::DeserializeError::MoreDataExpected(missing));
            }

            let (read, v0) = ::bebop::SubRecord::_deserialize_chained(&raw[i..])?;
            i += read;

            Ok((i, Self { delta: v0 }))
        }
    }

    impl<'raw> ::bebop::Record<'raw> for DeltaGrid {}
}
