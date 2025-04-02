// Automatically generated rust module for 'geo.proto' file

#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(unused_imports)]
#![allow(unknown_lints)]
#![allow(clippy::all)]
#![cfg_attr(rustfmt, rustfmt_skip)]


use std::borrow::Cow;
use quick_protobuf::{MessageInfo, MessageRead, MessageWrite, BytesReader, Writer, WriterBackend, Result};
use core::convert::{TryFrom, TryInto};
use quick_protobuf::sizeofs::*;
use super::*;

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Domain<'a> {
    pub type_pb: geo::mod_Domain::Type,
    pub value: Cow<'a, str>,
    pub attribute: Vec<geo::mod_Domain::Attribute<'a>>,
}

impl<'a> MessageRead<'a> for Domain<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(8) => msg.type_pb = r.read_enum(bytes)?,
                Ok(18) => msg.value = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(26) => msg.attribute.push(r.read_message::<geo::mod_Domain::Attribute>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Domain<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.type_pb == geo::mod_Domain::Type::Plain { 0 } else { 1 + sizeof_varint(*(&self.type_pb) as u64) }
        + if self.value == "" { 0 } else { 1 + sizeof_len((&self.value).len()) }
        + self.attribute.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.type_pb != geo::mod_Domain::Type::Plain { w.write_with_tag(8, |w| w.write_enum(*&self.type_pb as i32))?; }
        if self.value != "" { w.write_with_tag(18, |w| w.write_string(&**&self.value))?; }
        for s in &self.attribute { w.write_with_tag(26, |w| w.write_message(s))?; }
        Ok(())
    }
}


            // IMPORTANT: For any future changes, note that the lifetime parameter
            // of the `proto` field is set to 'static!!!
            //
            // This means that the internals of `proto` should at no point create a
            // mutable reference to something using that lifetime parameter, on pain
            // of UB. This applies even though it may be transmuted to a smaller
            // lifetime later (through `proto()` or `proto_mut()`).
            //
            // At the time of writing, the only possible thing that uses the
            // lifetime parameter is `Cow<'a, T>`, which never does this, so it's
            // not UB.
            //
            #[derive(Debug)]
            struct DomainOwnedInner {
                buf: Vec<u8>,
                proto: Option<Domain<'static>>,
                _pin: core::marker::PhantomPinned,
            }

            impl DomainOwnedInner {
                fn new(buf: Vec<u8>) -> Result<core::pin::Pin<Box<Self>>> {
                    let inner = Self {
                        buf,
                        proto: None,
                        _pin: core::marker::PhantomPinned,
                    };
                    let mut pinned = Box::pin(inner);

                    let mut reader = BytesReader::from_bytes(&pinned.buf);
                    let proto = Domain::from_reader(&mut reader, &pinned.buf)?;

                    unsafe {
                        let proto = core::mem::transmute::<_, Domain<'_>>(proto);
                        pinned.as_mut().get_unchecked_mut().proto = Some(proto);
                    }
                    Ok(pinned)
                }
            }

            pub struct DomainOwned {
                inner: core::pin::Pin<Box<DomainOwnedInner>>,
            }

            #[allow(dead_code)]
            impl DomainOwned {
                pub fn buf(&self) -> &[u8] {
                    &self.inner.buf
                }

                pub fn proto<'a>(&'a self) -> &'a Domain<'a> {
                    let proto = self.inner.proto.as_ref().unwrap();
                    unsafe { core::mem::transmute::<&Domain<'static>, &Domain<'a>>(proto) }
                }

                pub fn proto_mut<'a>(&'a mut self) -> &'a mut Domain<'a> {
                    let inner = self.inner.as_mut();
                    let inner = unsafe { inner.get_unchecked_mut() };
                    let proto = inner.proto.as_mut().unwrap();
                    unsafe { core::mem::transmute::<_, &mut Domain<'a>>(proto) }
                }
            }

            impl core::fmt::Debug for DomainOwned {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    self.inner.proto.as_ref().unwrap().fmt(f)
                }
            }

            impl TryFrom<Vec<u8>> for DomainOwned {
                type Error=quick_protobuf::Error;

                fn try_from(buf: Vec<u8>) -> Result<Self> {
                    Ok(Self { inner: DomainOwnedInner::new(buf)? })
                }
            }

            impl TryInto<Vec<u8>> for DomainOwned {
                type Error=quick_protobuf::Error;

                fn try_into(self) -> Result<Vec<u8>> {
                    let mut buf = Vec::new();
                    let mut writer = Writer::new(&mut buf);
                    self.inner.proto.as_ref().unwrap().write_message(&mut writer)?;
                    Ok(buf)
                }
            }

            impl From<Domain<'static>> for DomainOwned {
                fn from(proto: Domain<'static>) -> Self {
                    Self {
                        inner: Box::pin(DomainOwnedInner {
                            buf: Vec::new(),
                            proto: Some(proto),
                            _pin: core::marker::PhantomPinned,
                        })
                    }
                }
            }
            
pub mod mod_Domain {

use std::borrow::Cow;
use super::*;

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct Attribute<'a> {
    pub key: Cow<'a, str>,
    pub typed_value: geo::mod_Domain::mod_Attribute::OneOftyped_value,
}

impl<'a> MessageRead<'a> for Attribute<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.key = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(16) => msg.typed_value = geo::mod_Domain::mod_Attribute::OneOftyped_value::bool_value(r.read_bool(bytes)?),
                Ok(24) => msg.typed_value = geo::mod_Domain::mod_Attribute::OneOftyped_value::int_value(r.read_int64(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for Attribute<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.key == "" { 0 } else { 1 + sizeof_len((&self.key).len()) }
        + match self.typed_value {
            geo::mod_Domain::mod_Attribute::OneOftyped_value::bool_value(ref m) => 1 + sizeof_varint(*(m) as u64),
            geo::mod_Domain::mod_Attribute::OneOftyped_value::int_value(ref m) => 1 + sizeof_varint(*(m) as u64),
            geo::mod_Domain::mod_Attribute::OneOftyped_value::None => 0,
    }    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.key != "" { w.write_with_tag(10, |w| w.write_string(&**&self.key))?; }
        match self.typed_value {            geo::mod_Domain::mod_Attribute::OneOftyped_value::bool_value(ref m) => { w.write_with_tag(16, |w| w.write_bool(*m))? },
            geo::mod_Domain::mod_Attribute::OneOftyped_value::int_value(ref m) => { w.write_with_tag(24, |w| w.write_int64(*m))? },
            geo::mod_Domain::mod_Attribute::OneOftyped_value::None => {},
    }        Ok(())
    }
}


            // IMPORTANT: For any future changes, note that the lifetime parameter
            // of the `proto` field is set to 'static!!!
            //
            // This means that the internals of `proto` should at no point create a
            // mutable reference to something using that lifetime parameter, on pain
            // of UB. This applies even though it may be transmuted to a smaller
            // lifetime later (through `proto()` or `proto_mut()`).
            //
            // At the time of writing, the only possible thing that uses the
            // lifetime parameter is `Cow<'a, T>`, which never does this, so it's
            // not UB.
            //
            #[derive(Debug)]
            struct AttributeOwnedInner {
                buf: Vec<u8>,
                proto: Option<Attribute<'static>>,
                _pin: core::marker::PhantomPinned,
            }

            impl AttributeOwnedInner {
                fn new(buf: Vec<u8>) -> Result<core::pin::Pin<Box<Self>>> {
                    let inner = Self {
                        buf,
                        proto: None,
                        _pin: core::marker::PhantomPinned,
                    };
                    let mut pinned = Box::pin(inner);

                    let mut reader = BytesReader::from_bytes(&pinned.buf);
                    let proto = Attribute::from_reader(&mut reader, &pinned.buf)?;

                    unsafe {
                        let proto = core::mem::transmute::<_, Attribute<'_>>(proto);
                        pinned.as_mut().get_unchecked_mut().proto = Some(proto);
                    }
                    Ok(pinned)
                }
            }

            pub struct AttributeOwned {
                inner: core::pin::Pin<Box<AttributeOwnedInner>>,
            }

            #[allow(dead_code)]
            impl AttributeOwned {
                pub fn buf(&self) -> &[u8] {
                    &self.inner.buf
                }

                pub fn proto<'a>(&'a self) -> &'a Attribute<'a> {
                    let proto = self.inner.proto.as_ref().unwrap();
                    unsafe { core::mem::transmute::<&Attribute<'static>, &Attribute<'a>>(proto) }
                }

                pub fn proto_mut<'a>(&'a mut self) -> &'a mut Attribute<'a> {
                    let inner = self.inner.as_mut();
                    let inner = unsafe { inner.get_unchecked_mut() };
                    let proto = inner.proto.as_mut().unwrap();
                    unsafe { core::mem::transmute::<_, &mut Attribute<'a>>(proto) }
                }
            }

            impl core::fmt::Debug for AttributeOwned {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    self.inner.proto.as_ref().unwrap().fmt(f)
                }
            }

            impl TryFrom<Vec<u8>> for AttributeOwned {
                type Error=quick_protobuf::Error;

                fn try_from(buf: Vec<u8>) -> Result<Self> {
                    Ok(Self { inner: AttributeOwnedInner::new(buf)? })
                }
            }

            impl TryInto<Vec<u8>> for AttributeOwned {
                type Error=quick_protobuf::Error;

                fn try_into(self) -> Result<Vec<u8>> {
                    let mut buf = Vec::new();
                    let mut writer = Writer::new(&mut buf);
                    self.inner.proto.as_ref().unwrap().write_message(&mut writer)?;
                    Ok(buf)
                }
            }

            impl From<Attribute<'static>> for AttributeOwned {
                fn from(proto: Attribute<'static>) -> Self {
                    Self {
                        inner: Box::pin(AttributeOwnedInner {
                            buf: Vec::new(),
                            proto: Some(proto),
                            _pin: core::marker::PhantomPinned,
                        })
                    }
                }
            }
            
pub mod mod_Attribute {

use super::*;

#[derive(Debug, PartialEq, Clone)]
pub enum OneOftyped_value {
    bool_value(bool),
    int_value(i64),
    None,
}

impl Default for OneOftyped_value {
    fn default() -> Self {
        OneOftyped_value::None
    }
}

}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Type {
    Plain = 0,
    Regex = 1,
    Domain = 2,
    Full = 3,
}

impl Default for Type {
    fn default() -> Self {
        Type::Plain
    }
}

impl From<i32> for Type {
    fn from(i: i32) -> Self {
        match i {
            0 => Type::Plain,
            1 => Type::Regex,
            2 => Type::Domain,
            3 => Type::Full,
            _ => Self::default(),
        }
    }
}

impl<'a> From<&'a str> for Type {
    fn from(s: &'a str) -> Self {
        match s {
            "Plain" => Type::Plain,
            "Regex" => Type::Regex,
            "Domain" => Type::Domain,
            "Full" => Type::Full,
            _ => Self::default(),
        }
    }
}

}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct CIDR<'a> {
    pub ip: Cow<'a, [u8]>,
    pub prefix: u32,
}

impl<'a> MessageRead<'a> for CIDR<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.ip = r.read_bytes(bytes).map(Cow::Borrowed)?,
                Ok(16) => msg.prefix = r.read_uint32(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for CIDR<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.ip == Cow::Borrowed(b"") { 0 } else { 1 + sizeof_len((&self.ip).len()) }
        + if self.prefix == 0u32 { 0 } else { 1 + sizeof_varint(*(&self.prefix) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.ip != Cow::Borrowed(b"") { w.write_with_tag(10, |w| w.write_bytes(&**&self.ip))?; }
        if self.prefix != 0u32 { w.write_with_tag(16, |w| w.write_uint32(*&self.prefix))?; }
        Ok(())
    }
}


            // IMPORTANT: For any future changes, note that the lifetime parameter
            // of the `proto` field is set to 'static!!!
            //
            // This means that the internals of `proto` should at no point create a
            // mutable reference to something using that lifetime parameter, on pain
            // of UB. This applies even though it may be transmuted to a smaller
            // lifetime later (through `proto()` or `proto_mut()`).
            //
            // At the time of writing, the only possible thing that uses the
            // lifetime parameter is `Cow<'a, T>`, which never does this, so it's
            // not UB.
            //
            #[derive(Debug)]
            struct CIDROwnedInner {
                buf: Vec<u8>,
                proto: Option<CIDR<'static>>,
                _pin: core::marker::PhantomPinned,
            }

            impl CIDROwnedInner {
                fn new(buf: Vec<u8>) -> Result<core::pin::Pin<Box<Self>>> {
                    let inner = Self {
                        buf,
                        proto: None,
                        _pin: core::marker::PhantomPinned,
                    };
                    let mut pinned = Box::pin(inner);

                    let mut reader = BytesReader::from_bytes(&pinned.buf);
                    let proto = CIDR::from_reader(&mut reader, &pinned.buf)?;

                    unsafe {
                        let proto = core::mem::transmute::<_, CIDR<'_>>(proto);
                        pinned.as_mut().get_unchecked_mut().proto = Some(proto);
                    }
                    Ok(pinned)
                }
            }

            pub struct CIDROwned {
                inner: core::pin::Pin<Box<CIDROwnedInner>>,
            }

            #[allow(dead_code)]
            impl CIDROwned {
                pub fn buf(&self) -> &[u8] {
                    &self.inner.buf
                }

                pub fn proto<'a>(&'a self) -> &'a CIDR<'a> {
                    let proto = self.inner.proto.as_ref().unwrap();
                    unsafe { core::mem::transmute::<&CIDR<'static>, &CIDR<'a>>(proto) }
                }

                pub fn proto_mut<'a>(&'a mut self) -> &'a mut CIDR<'a> {
                    let inner = self.inner.as_mut();
                    let inner = unsafe { inner.get_unchecked_mut() };
                    let proto = inner.proto.as_mut().unwrap();
                    unsafe { core::mem::transmute::<_, &mut CIDR<'a>>(proto) }
                }
            }

            impl core::fmt::Debug for CIDROwned {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    self.inner.proto.as_ref().unwrap().fmt(f)
                }
            }

            impl TryFrom<Vec<u8>> for CIDROwned {
                type Error=quick_protobuf::Error;

                fn try_from(buf: Vec<u8>) -> Result<Self> {
                    Ok(Self { inner: CIDROwnedInner::new(buf)? })
                }
            }

            impl TryInto<Vec<u8>> for CIDROwned {
                type Error=quick_protobuf::Error;

                fn try_into(self) -> Result<Vec<u8>> {
                    let mut buf = Vec::new();
                    let mut writer = Writer::new(&mut buf);
                    self.inner.proto.as_ref().unwrap().write_message(&mut writer)?;
                    Ok(buf)
                }
            }

            impl From<CIDR<'static>> for CIDROwned {
                fn from(proto: CIDR<'static>) -> Self {
                    Self {
                        inner: Box::pin(CIDROwnedInner {
                            buf: Vec::new(),
                            proto: Some(proto),
                            _pin: core::marker::PhantomPinned,
                        })
                    }
                }
            }
            
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct GeoIP<'a> {
    pub country_code: Cow<'a, str>,
    pub cidr: Vec<geo::CIDR<'a>>,
    pub reverse_match: bool,
}

impl<'a> MessageRead<'a> for GeoIP<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.country_code = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(18) => msg.cidr.push(r.read_message::<geo::CIDR>(bytes)?),
                Ok(24) => msg.reverse_match = r.read_bool(bytes)?,
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for GeoIP<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.country_code == "" { 0 } else { 1 + sizeof_len((&self.country_code).len()) }
        + self.cidr.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
        + if self.reverse_match == false { 0 } else { 1 + sizeof_varint(*(&self.reverse_match) as u64) }
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.country_code != "" { w.write_with_tag(10, |w| w.write_string(&**&self.country_code))?; }
        for s in &self.cidr { w.write_with_tag(18, |w| w.write_message(s))?; }
        if self.reverse_match != false { w.write_with_tag(24, |w| w.write_bool(*&self.reverse_match))?; }
        Ok(())
    }
}


            // IMPORTANT: For any future changes, note that the lifetime parameter
            // of the `proto` field is set to 'static!!!
            //
            // This means that the internals of `proto` should at no point create a
            // mutable reference to something using that lifetime parameter, on pain
            // of UB. This applies even though it may be transmuted to a smaller
            // lifetime later (through `proto()` or `proto_mut()`).
            //
            // At the time of writing, the only possible thing that uses the
            // lifetime parameter is `Cow<'a, T>`, which never does this, so it's
            // not UB.
            //
            #[derive(Debug)]
            struct GeoIPOwnedInner {
                buf: Vec<u8>,
                proto: Option<GeoIP<'static>>,
                _pin: core::marker::PhantomPinned,
            }

            impl GeoIPOwnedInner {
                fn new(buf: Vec<u8>) -> Result<core::pin::Pin<Box<Self>>> {
                    let inner = Self {
                        buf,
                        proto: None,
                        _pin: core::marker::PhantomPinned,
                    };
                    let mut pinned = Box::pin(inner);

                    let mut reader = BytesReader::from_bytes(&pinned.buf);
                    let proto = GeoIP::from_reader(&mut reader, &pinned.buf)?;

                    unsafe {
                        let proto = core::mem::transmute::<_, GeoIP<'_>>(proto);
                        pinned.as_mut().get_unchecked_mut().proto = Some(proto);
                    }
                    Ok(pinned)
                }
            }

            pub struct GeoIPOwned {
                inner: core::pin::Pin<Box<GeoIPOwnedInner>>,
            }

            #[allow(dead_code)]
            impl GeoIPOwned {
                pub fn buf(&self) -> &[u8] {
                    &self.inner.buf
                }

                pub fn proto<'a>(&'a self) -> &'a GeoIP<'a> {
                    let proto = self.inner.proto.as_ref().unwrap();
                    unsafe { core::mem::transmute::<&GeoIP<'static>, &GeoIP<'a>>(proto) }
                }

                pub fn proto_mut<'a>(&'a mut self) -> &'a mut GeoIP<'a> {
                    let inner = self.inner.as_mut();
                    let inner = unsafe { inner.get_unchecked_mut() };
                    let proto = inner.proto.as_mut().unwrap();
                    unsafe { core::mem::transmute::<_, &mut GeoIP<'a>>(proto) }
                }
            }

            impl core::fmt::Debug for GeoIPOwned {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    self.inner.proto.as_ref().unwrap().fmt(f)
                }
            }

            impl TryFrom<Vec<u8>> for GeoIPOwned {
                type Error=quick_protobuf::Error;

                fn try_from(buf: Vec<u8>) -> Result<Self> {
                    Ok(Self { inner: GeoIPOwnedInner::new(buf)? })
                }
            }

            impl TryInto<Vec<u8>> for GeoIPOwned {
                type Error=quick_protobuf::Error;

                fn try_into(self) -> Result<Vec<u8>> {
                    let mut buf = Vec::new();
                    let mut writer = Writer::new(&mut buf);
                    self.inner.proto.as_ref().unwrap().write_message(&mut writer)?;
                    Ok(buf)
                }
            }

            impl From<GeoIP<'static>> for GeoIPOwned {
                fn from(proto: GeoIP<'static>) -> Self {
                    Self {
                        inner: Box::pin(GeoIPOwnedInner {
                            buf: Vec::new(),
                            proto: Some(proto),
                            _pin: core::marker::PhantomPinned,
                        })
                    }
                }
            }
            
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct GeoIPList<'a> {
    pub entry: Vec<geo::GeoIP<'a>>,
}

impl<'a> MessageRead<'a> for GeoIPList<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.entry.push(r.read_message::<geo::GeoIP>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for GeoIPList<'a> {
    fn get_size(&self) -> usize {
        0
        + self.entry.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        for s in &self.entry { w.write_with_tag(10, |w| w.write_message(s))?; }
        Ok(())
    }
}


            // IMPORTANT: For any future changes, note that the lifetime parameter
            // of the `proto` field is set to 'static!!!
            //
            // This means that the internals of `proto` should at no point create a
            // mutable reference to something using that lifetime parameter, on pain
            // of UB. This applies even though it may be transmuted to a smaller
            // lifetime later (through `proto()` or `proto_mut()`).
            //
            // At the time of writing, the only possible thing that uses the
            // lifetime parameter is `Cow<'a, T>`, which never does this, so it's
            // not UB.
            //
            #[derive(Debug)]
            struct GeoIPListOwnedInner {
                buf: Vec<u8>,
                proto: Option<GeoIPList<'static>>,
                _pin: core::marker::PhantomPinned,
            }

            impl GeoIPListOwnedInner {
                fn new(buf: Vec<u8>) -> Result<core::pin::Pin<Box<Self>>> {
                    let inner = Self {
                        buf,
                        proto: None,
                        _pin: core::marker::PhantomPinned,
                    };
                    let mut pinned = Box::pin(inner);

                    let mut reader = BytesReader::from_bytes(&pinned.buf);
                    let proto = GeoIPList::from_reader(&mut reader, &pinned.buf)?;

                    unsafe {
                        let proto = core::mem::transmute::<_, GeoIPList<'_>>(proto);
                        pinned.as_mut().get_unchecked_mut().proto = Some(proto);
                    }
                    Ok(pinned)
                }
            }

            pub struct GeoIPListOwned {
                inner: core::pin::Pin<Box<GeoIPListOwnedInner>>,
            }

            #[allow(dead_code)]
            impl GeoIPListOwned {
                pub fn buf(&self) -> &[u8] {
                    &self.inner.buf
                }

                pub fn proto<'a>(&'a self) -> &'a GeoIPList<'a> {
                    let proto = self.inner.proto.as_ref().unwrap();
                    unsafe { core::mem::transmute::<&GeoIPList<'static>, &GeoIPList<'a>>(proto) }
                }

                pub fn proto_mut<'a>(&'a mut self) -> &'a mut GeoIPList<'a> {
                    let inner = self.inner.as_mut();
                    let inner = unsafe { inner.get_unchecked_mut() };
                    let proto = inner.proto.as_mut().unwrap();
                    unsafe { core::mem::transmute::<_, &mut GeoIPList<'a>>(proto) }
                }
            }

            impl core::fmt::Debug for GeoIPListOwned {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    self.inner.proto.as_ref().unwrap().fmt(f)
                }
            }

            impl TryFrom<Vec<u8>> for GeoIPListOwned {
                type Error=quick_protobuf::Error;

                fn try_from(buf: Vec<u8>) -> Result<Self> {
                    Ok(Self { inner: GeoIPListOwnedInner::new(buf)? })
                }
            }

            impl TryInto<Vec<u8>> for GeoIPListOwned {
                type Error=quick_protobuf::Error;

                fn try_into(self) -> Result<Vec<u8>> {
                    let mut buf = Vec::new();
                    let mut writer = Writer::new(&mut buf);
                    self.inner.proto.as_ref().unwrap().write_message(&mut writer)?;
                    Ok(buf)
                }
            }

            impl From<GeoIPList<'static>> for GeoIPListOwned {
                fn from(proto: GeoIPList<'static>) -> Self {
                    Self {
                        inner: Box::pin(GeoIPListOwnedInner {
                            buf: Vec::new(),
                            proto: Some(proto),
                            _pin: core::marker::PhantomPinned,
                        })
                    }
                }
            }
            
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct GeoSite<'a> {
    pub country_code: Cow<'a, str>,
    pub domain: Vec<geo::Domain<'a>>,
}

impl<'a> MessageRead<'a> for GeoSite<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.country_code = r.read_string(bytes).map(Cow::Borrowed)?,
                Ok(18) => msg.domain.push(r.read_message::<geo::Domain>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for GeoSite<'a> {
    fn get_size(&self) -> usize {
        0
        + if self.country_code == "" { 0 } else { 1 + sizeof_len((&self.country_code).len()) }
        + self.domain.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        if self.country_code != "" { w.write_with_tag(10, |w| w.write_string(&**&self.country_code))?; }
        for s in &self.domain { w.write_with_tag(18, |w| w.write_message(s))?; }
        Ok(())
    }
}


            // IMPORTANT: For any future changes, note that the lifetime parameter
            // of the `proto` field is set to 'static!!!
            //
            // This means that the internals of `proto` should at no point create a
            // mutable reference to something using that lifetime parameter, on pain
            // of UB. This applies even though it may be transmuted to a smaller
            // lifetime later (through `proto()` or `proto_mut()`).
            //
            // At the time of writing, the only possible thing that uses the
            // lifetime parameter is `Cow<'a, T>`, which never does this, so it's
            // not UB.
            //
            #[derive(Debug)]
            struct GeoSiteOwnedInner {
                buf: Vec<u8>,
                proto: Option<GeoSite<'static>>,
                _pin: core::marker::PhantomPinned,
            }

            impl GeoSiteOwnedInner {
                fn new(buf: Vec<u8>) -> Result<core::pin::Pin<Box<Self>>> {
                    let inner = Self {
                        buf,
                        proto: None,
                        _pin: core::marker::PhantomPinned,
                    };
                    let mut pinned = Box::pin(inner);

                    let mut reader = BytesReader::from_bytes(&pinned.buf);
                    let proto = GeoSite::from_reader(&mut reader, &pinned.buf)?;

                    unsafe {
                        let proto = core::mem::transmute::<_, GeoSite<'_>>(proto);
                        pinned.as_mut().get_unchecked_mut().proto = Some(proto);
                    }
                    Ok(pinned)
                }
            }

            pub struct GeoSiteOwned {
                inner: core::pin::Pin<Box<GeoSiteOwnedInner>>,
            }

            #[allow(dead_code)]
            impl GeoSiteOwned {
                pub fn buf(&self) -> &[u8] {
                    &self.inner.buf
                }

                pub fn proto<'a>(&'a self) -> &'a GeoSite<'a> {
                    let proto = self.inner.proto.as_ref().unwrap();
                    unsafe { core::mem::transmute::<&GeoSite<'static>, &GeoSite<'a>>(proto) }
                }

                pub fn proto_mut<'a>(&'a mut self) -> &'a mut GeoSite<'a> {
                    let inner = self.inner.as_mut();
                    let inner = unsafe { inner.get_unchecked_mut() };
                    let proto = inner.proto.as_mut().unwrap();
                    unsafe { core::mem::transmute::<_, &mut GeoSite<'a>>(proto) }
                }
            }

            impl core::fmt::Debug for GeoSiteOwned {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    self.inner.proto.as_ref().unwrap().fmt(f)
                }
            }

            impl TryFrom<Vec<u8>> for GeoSiteOwned {
                type Error=quick_protobuf::Error;

                fn try_from(buf: Vec<u8>) -> Result<Self> {
                    Ok(Self { inner: GeoSiteOwnedInner::new(buf)? })
                }
            }

            impl TryInto<Vec<u8>> for GeoSiteOwned {
                type Error=quick_protobuf::Error;

                fn try_into(self) -> Result<Vec<u8>> {
                    let mut buf = Vec::new();
                    let mut writer = Writer::new(&mut buf);
                    self.inner.proto.as_ref().unwrap().write_message(&mut writer)?;
                    Ok(buf)
                }
            }

            impl From<GeoSite<'static>> for GeoSiteOwned {
                fn from(proto: GeoSite<'static>) -> Self {
                    Self {
                        inner: Box::pin(GeoSiteOwnedInner {
                            buf: Vec::new(),
                            proto: Some(proto),
                            _pin: core::marker::PhantomPinned,
                        })
                    }
                }
            }
            
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Debug, Default, PartialEq, Clone)]
pub struct GeoSiteList<'a> {
    pub entry: Vec<geo::GeoSite<'a>>,
}

impl<'a> MessageRead<'a> for GeoSiteList<'a> {
    fn from_reader(r: &mut BytesReader, bytes: &'a [u8]) -> Result<Self> {
        let mut msg = Self::default();
        while !r.is_eof() {
            match r.next_tag(bytes) {
                Ok(10) => msg.entry.push(r.read_message::<geo::GeoSite>(bytes)?),
                Ok(t) => { r.read_unknown(bytes, t)?; }
                Err(e) => return Err(e),
            }
        }
        Ok(msg)
    }
}

impl<'a> MessageWrite for GeoSiteList<'a> {
    fn get_size(&self) -> usize {
        0
        + self.entry.iter().map(|s| 1 + sizeof_len((s).get_size())).sum::<usize>()
    }

    fn write_message<W: WriterBackend>(&self, w: &mut Writer<W>) -> Result<()> {
        for s in &self.entry { w.write_with_tag(10, |w| w.write_message(s))?; }
        Ok(())
    }
}


            // IMPORTANT: For any future changes, note that the lifetime parameter
            // of the `proto` field is set to 'static!!!
            //
            // This means that the internals of `proto` should at no point create a
            // mutable reference to something using that lifetime parameter, on pain
            // of UB. This applies even though it may be transmuted to a smaller
            // lifetime later (through `proto()` or `proto_mut()`).
            //
            // At the time of writing, the only possible thing that uses the
            // lifetime parameter is `Cow<'a, T>`, which never does this, so it's
            // not UB.
            //
            #[derive(Debug)]
            struct GeoSiteListOwnedInner {
                buf: Vec<u8>,
                proto: Option<GeoSiteList<'static>>,
                _pin: core::marker::PhantomPinned,
            }

            impl GeoSiteListOwnedInner {
                fn new(buf: Vec<u8>) -> Result<core::pin::Pin<Box<Self>>> {
                    let inner = Self {
                        buf,
                        proto: None,
                        _pin: core::marker::PhantomPinned,
                    };
                    let mut pinned = Box::pin(inner);

                    let mut reader = BytesReader::from_bytes(&pinned.buf);
                    let proto = GeoSiteList::from_reader(&mut reader, &pinned.buf)?;

                    unsafe {
                        let proto = core::mem::transmute::<_, GeoSiteList<'_>>(proto);
                        pinned.as_mut().get_unchecked_mut().proto = Some(proto);
                    }
                    Ok(pinned)
                }
            }

            pub struct GeoSiteListOwned {
                inner: core::pin::Pin<Box<GeoSiteListOwnedInner>>,
            }

            #[allow(dead_code)]
            impl GeoSiteListOwned {
                pub fn buf(&self) -> &[u8] {
                    &self.inner.buf
                }

                pub fn proto<'a>(&'a self) -> &'a GeoSiteList<'a> {
                    let proto = self.inner.proto.as_ref().unwrap();
                    unsafe { core::mem::transmute::<&GeoSiteList<'static>, &GeoSiteList<'a>>(proto) }
                }

                pub fn proto_mut<'a>(&'a mut self) -> &'a mut GeoSiteList<'a> {
                    let inner = self.inner.as_mut();
                    let inner = unsafe { inner.get_unchecked_mut() };
                    let proto = inner.proto.as_mut().unwrap();
                    unsafe { core::mem::transmute::<_, &mut GeoSiteList<'a>>(proto) }
                }
            }

            impl core::fmt::Debug for GeoSiteListOwned {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                    self.inner.proto.as_ref().unwrap().fmt(f)
                }
            }

            impl TryFrom<Vec<u8>> for GeoSiteListOwned {
                type Error=quick_protobuf::Error;

                fn try_from(buf: Vec<u8>) -> Result<Self> {
                    Ok(Self { inner: GeoSiteListOwnedInner::new(buf)? })
                }
            }

            impl TryInto<Vec<u8>> for GeoSiteListOwned {
                type Error=quick_protobuf::Error;

                fn try_into(self) -> Result<Vec<u8>> {
                    let mut buf = Vec::new();
                    let mut writer = Writer::new(&mut buf);
                    self.inner.proto.as_ref().unwrap().write_message(&mut writer)?;
                    Ok(buf)
                }
            }

            impl From<GeoSiteList<'static>> for GeoSiteListOwned {
                fn from(proto: GeoSiteList<'static>) -> Self {
                    Self {
                        inner: Box::pin(GeoSiteListOwnedInner {
                            buf: Vec::new(),
                            proto: Some(proto),
                            _pin: core::marker::PhantomPinned,
                        })
                    }
                }
            }
            
