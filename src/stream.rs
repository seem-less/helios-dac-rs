use std::io;

use crate::{Point, DeviceStatus};

/// A bi-directional communication stream between the user and a `Dac`.
pub struct Stream {
    /// An up-to-date representation of the `DAC` with which the stream is connected.
    dac: Addressed,
    /// A buffer to re-use for queueing commands via the `queue_commands` method.
    command_buffer: Vec<QueuedCommand>,
    /// A buffer to re-use for queueing points for `Data` commands.
    point_buffer: Vec<Point>,
    /// A buffer used for efficiently writing and reading bytes to and from TCP.
    bytes: Vec<u8>,
}

/// A runtime representation of any of the possible commands.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum QueuedCommand {
    PrepareStream,
    Begin(protocol::command::Begin),
    PointRate(protocol::command::PointRate),
    Data(ops::Range<usize>),
    Stop,
    EmergencyStop,
    ClearEmergencyStop,
    Ping,
}

impl Stream {
    fn send_command<C>(&mut self, command: C) -> io::Result<()>
    where
        C: Command + WriteToBytes,
    {
        let Stream {
            ref mut bytes,
            ..
        } = *self;
        send_command(bytes, command)
    }

    fn recv_response(&mut self, expected_command: u8) -> Result<(), CommunicationError> {
        let Stream {
            ref mut bytes,
            ref mut dac,
            ..
        } = *self;
        recv_response(bytes, dac, expected_command)
    }

    /// Borrow the inner DAC to examine its state.
    pub fn dac(&self) -> &Addressed {
        &self.dac
    }

    /// Queue one or more commands to be submitted to the DAC at once.
    pub fn queue_commands(&mut self) -> CommandQueue {
        self.command_buffer.clear();
        self.point_buffer.clear();
        CommandQueue { stream: self }
    }
}

/// A queue of commands that are to be submitted at once before listening for their responses.
pub struct CommandQueue<'a> {
    stream: &'a mut Stream,
}

/// A DAC along with its ID.
///
/// This type can be used as though it is a `Dac` in many places as it implements
/// `Deref<Target = Dac>`
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Addressed {
    /// This may be used to distinguish between multiple DACs broadcasting on a network.
    pub id: u32,
    /// The state of the DAC itself.
    pub dac: Dac,
}

/// A simple abstraction around a single Helios DAC.
///
/// This type monitors the multiple state machines described within the protocol and provides
/// access to information about the Ether Dream hardware and software implementations.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Dac {
    /// Firmware version
    pub sw_revision: u32,
    /// The maximum number of points that the DAC may buffer at once.
    pub buffer_capacity: u32,
    /// The maximum rate at which the DAC may process buffered points.
    pub max_point_rate: u32,
    /// Whether the DAC is ready to recieve a frame or not
    pub status: DeviceStatus,
}