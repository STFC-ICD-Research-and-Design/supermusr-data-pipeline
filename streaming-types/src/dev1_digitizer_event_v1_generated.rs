// automatically generated by the FlatBuffers compiler, do not modify


// @generated

use crate::frame_metadata_v1_generated::*;
use core::mem;
use core::cmp::Ordering;

extern crate flatbuffers;
use self::flatbuffers::{EndianScalar, Follow};

pub enum DigitizerEventListMessageOffset {}
#[derive(Copy, Clone, PartialEq)]

pub struct DigitizerEventListMessage<'a> {
  pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for DigitizerEventListMessage<'a> {
  type Inner = DigitizerEventListMessage<'a>;
  #[inline]
  unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
    Self { _tab: flatbuffers::Table::new(buf, loc) }
  }
}

impl<'a> DigitizerEventListMessage<'a> {
  pub const VT_DIGITIZER_ID: flatbuffers::VOffsetT = 4;
  pub const VT_METADATA: flatbuffers::VOffsetT = 6;
  pub const VT_TIME: flatbuffers::VOffsetT = 8;
  pub const VT_VOLTAGE: flatbuffers::VOffsetT = 10;
  pub const VT_CHANNEL: flatbuffers::VOffsetT = 12;

  #[inline]
  pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
    DigitizerEventListMessage { _tab: table }
  }
  #[allow(unused_mut)]
  pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr>(
    _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr>,
    args: &'args DigitizerEventListMessageArgs<'args>
  ) -> flatbuffers::WIPOffset<DigitizerEventListMessage<'bldr>> {
    let mut builder = DigitizerEventListMessageBuilder::new(_fbb);
    if let Some(x) = args.channel { builder.add_channel(x); }
    if let Some(x) = args.voltage { builder.add_voltage(x); }
    if let Some(x) = args.time { builder.add_time(x); }
    if let Some(x) = args.metadata { builder.add_metadata(x); }
    builder.add_digitizer_id(args.digitizer_id);
    builder.finish()
  }


  #[inline]
  pub fn digitizer_id(&self) -> u8 {
    // Safety:
    // Created from valid Table for this object
    // which contains a valid value in this slot
    unsafe { self._tab.get::<u8>(DigitizerEventListMessage::VT_DIGITIZER_ID, Some(0)).unwrap()}
  }
  #[inline]
  pub fn metadata(&self) -> FrameMetadataV1<'a> {
    // Safety:
    // Created from valid Table for this object
    // which contains a valid value in this slot
    unsafe { self._tab.get::<flatbuffers::ForwardsUOffset<FrameMetadataV1>>(DigitizerEventListMessage::VT_METADATA, None).unwrap()}
  }
  #[inline]
  pub fn time(&self) -> Option<flatbuffers::Vector<'a, u32>> {
    // Safety:
    // Created from valid Table for this object
    // which contains a valid value in this slot
    unsafe { self._tab.get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'a, u32>>>(DigitizerEventListMessage::VT_TIME, None)}
  }
  #[inline]
  pub fn voltage(&self) -> Option<flatbuffers::Vector<'a, u16>> {
    // Safety:
    // Created from valid Table for this object
    // which contains a valid value in this slot
    unsafe { self._tab.get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'a, u16>>>(DigitizerEventListMessage::VT_VOLTAGE, None)}
  }
  #[inline]
  pub fn channel(&self) -> Option<flatbuffers::Vector<'a, u32>> {
    // Safety:
    // Created from valid Table for this object
    // which contains a valid value in this slot
    unsafe { self._tab.get::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'a, u32>>>(DigitizerEventListMessage::VT_CHANNEL, None)}
  }
}

impl flatbuffers::Verifiable for DigitizerEventListMessage<'_> {
  #[inline]
  fn run_verifier(
    v: &mut flatbuffers::Verifier, pos: usize
  ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
    use self::flatbuffers::Verifiable;
    v.visit_table(pos)?
     .visit_field::<u8>("digitizer_id", Self::VT_DIGITIZER_ID, false)?
     .visit_field::<flatbuffers::ForwardsUOffset<FrameMetadataV1>>("metadata", Self::VT_METADATA, true)?
     .visit_field::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'_, u32>>>("time", Self::VT_TIME, false)?
     .visit_field::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'_, u16>>>("voltage", Self::VT_VOLTAGE, false)?
     .visit_field::<flatbuffers::ForwardsUOffset<flatbuffers::Vector<'_, u32>>>("channel", Self::VT_CHANNEL, false)?
     .finish();
    Ok(())
  }
}
pub struct DigitizerEventListMessageArgs<'a> {
    pub digitizer_id: u8,
    pub metadata: Option<flatbuffers::WIPOffset<FrameMetadataV1<'a>>>,
    pub time: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, u32>>>,
    pub voltage: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, u16>>>,
    pub channel: Option<flatbuffers::WIPOffset<flatbuffers::Vector<'a, u32>>>,
}
impl<'a> Default for DigitizerEventListMessageArgs<'a> {
  #[inline]
  fn default() -> Self {
    DigitizerEventListMessageArgs {
      digitizer_id: 0,
      metadata: None, // required field
      time: None,
      voltage: None,
      channel: None,
    }
  }
}

pub struct DigitizerEventListMessageBuilder<'a: 'b, 'b> {
  fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a>,
  start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b> DigitizerEventListMessageBuilder<'a, 'b> {
  #[inline]
  pub fn add_digitizer_id(&mut self, digitizer_id: u8) {
    self.fbb_.push_slot::<u8>(DigitizerEventListMessage::VT_DIGITIZER_ID, digitizer_id, 0);
  }
  #[inline]
  pub fn add_metadata(&mut self, metadata: flatbuffers::WIPOffset<FrameMetadataV1<'b >>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<FrameMetadataV1>>(DigitizerEventListMessage::VT_METADATA, metadata);
  }
  #[inline]
  pub fn add_time(&mut self, time: flatbuffers::WIPOffset<flatbuffers::Vector<'b , u32>>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(DigitizerEventListMessage::VT_TIME, time);
  }
  #[inline]
  pub fn add_voltage(&mut self, voltage: flatbuffers::WIPOffset<flatbuffers::Vector<'b , u16>>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(DigitizerEventListMessage::VT_VOLTAGE, voltage);
  }
  #[inline]
  pub fn add_channel(&mut self, channel: flatbuffers::WIPOffset<flatbuffers::Vector<'b , u32>>) {
    self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(DigitizerEventListMessage::VT_CHANNEL, channel);
  }
  #[inline]
  pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>) -> DigitizerEventListMessageBuilder<'a, 'b> {
    let start = _fbb.start_table();
    DigitizerEventListMessageBuilder {
      fbb_: _fbb,
      start_: start,
    }
  }
  #[inline]
  pub fn finish(self) -> flatbuffers::WIPOffset<DigitizerEventListMessage<'a>> {
    let o = self.fbb_.end_table(self.start_);
    self.fbb_.required(o, DigitizerEventListMessage::VT_METADATA,"metadata");
    flatbuffers::WIPOffset::new(o.value())
  }
}

impl core::fmt::Debug for DigitizerEventListMessage<'_> {
  fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
    let mut ds = f.debug_struct("DigitizerEventListMessage");
      ds.field("digitizer_id", &self.digitizer_id());
      ds.field("metadata", &self.metadata());
      ds.field("time", &self.time());
      ds.field("voltage", &self.voltage());
      ds.field("channel", &self.channel());
      ds.finish()
  }
}
#[inline]
/// Verifies that a buffer of bytes contains a `DigitizerEventListMessage`
/// and returns it.
/// Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `root_as_digitizer_event_list_message_unchecked`.
pub fn root_as_digitizer_event_list_message(buf: &[u8]) -> Result<DigitizerEventListMessage, flatbuffers::InvalidFlatbuffer> {
  flatbuffers::root::<DigitizerEventListMessage>(buf)
}
#[inline]
/// Verifies that a buffer of bytes contains a size prefixed
/// `DigitizerEventListMessage` and returns it.
/// Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `size_prefixed_root_as_digitizer_event_list_message_unchecked`.
pub fn size_prefixed_root_as_digitizer_event_list_message(buf: &[u8]) -> Result<DigitizerEventListMessage, flatbuffers::InvalidFlatbuffer> {
  flatbuffers::size_prefixed_root::<DigitizerEventListMessage>(buf)
}
#[inline]
/// Verifies, with the given options, that a buffer of bytes
/// contains a `DigitizerEventListMessage` and returns it.
/// Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `root_as_digitizer_event_list_message_unchecked`.
pub fn root_as_digitizer_event_list_message_with_opts<'b, 'o>(
  opts: &'o flatbuffers::VerifierOptions,
  buf: &'b [u8],
) -> Result<DigitizerEventListMessage<'b>, flatbuffers::InvalidFlatbuffer> {
  flatbuffers::root_with_opts::<DigitizerEventListMessage<'b>>(opts, buf)
}
#[inline]
/// Verifies, with the given verifier options, that a buffer of
/// bytes contains a size prefixed `DigitizerEventListMessage` and returns
/// it. Note that verification is still experimental and may not
/// catch every error, or be maximally performant. For the
/// previous, unchecked, behavior use
/// `root_as_digitizer_event_list_message_unchecked`.
pub fn size_prefixed_root_as_digitizer_event_list_message_with_opts<'b, 'o>(
  opts: &'o flatbuffers::VerifierOptions,
  buf: &'b [u8],
) -> Result<DigitizerEventListMessage<'b>, flatbuffers::InvalidFlatbuffer> {
  flatbuffers::size_prefixed_root_with_opts::<DigitizerEventListMessage<'b>>(opts, buf)
}
#[inline]
/// Assumes, without verification, that a buffer of bytes contains a DigitizerEventListMessage and returns it.
/// # Safety
/// Callers must trust the given bytes do indeed contain a valid `DigitizerEventListMessage`.
pub unsafe fn root_as_digitizer_event_list_message_unchecked(buf: &[u8]) -> DigitizerEventListMessage {
  flatbuffers::root_unchecked::<DigitizerEventListMessage>(buf)
}
#[inline]
/// Assumes, without verification, that a buffer of bytes contains a size prefixed DigitizerEventListMessage and returns it.
/// # Safety
/// Callers must trust the given bytes do indeed contain a valid size prefixed `DigitizerEventListMessage`.
pub unsafe fn size_prefixed_root_as_digitizer_event_list_message_unchecked(buf: &[u8]) -> DigitizerEventListMessage {
  flatbuffers::size_prefixed_root_unchecked::<DigitizerEventListMessage>(buf)
}
pub const DIGITIZER_EVENT_LIST_MESSAGE_IDENTIFIER: &str = "dev1";

#[inline]
pub fn digitizer_event_list_message_buffer_has_identifier(buf: &[u8]) -> bool {
  flatbuffers::buffer_has_identifier(buf, DIGITIZER_EVENT_LIST_MESSAGE_IDENTIFIER, false)
}

#[inline]
pub fn digitizer_event_list_message_size_prefixed_buffer_has_identifier(buf: &[u8]) -> bool {
  flatbuffers::buffer_has_identifier(buf, DIGITIZER_EVENT_LIST_MESSAGE_IDENTIFIER, true)
}

#[inline]
pub fn finish_digitizer_event_list_message_buffer<'a, 'b>(
    fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>,
    root: flatbuffers::WIPOffset<DigitizerEventListMessage<'a>>) {
  fbb.finish(root, Some(DIGITIZER_EVENT_LIST_MESSAGE_IDENTIFIER));
}

#[inline]
pub fn finish_size_prefixed_digitizer_event_list_message_buffer<'a, 'b>(fbb: &'b mut flatbuffers::FlatBufferBuilder<'a>, root: flatbuffers::WIPOffset<DigitizerEventListMessage<'a>>) {
  fbb.finish_size_prefixed(root, Some(DIGITIZER_EVENT_LIST_MESSAGE_IDENTIFIER));
}
