#![no_std]

extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;
use alloc::boxed::Box;

use core::ops::Deref;
use core::ops::DerefMut;

use spin::RwLock;

pub type MsgResult<T> = Result<T, &'static str>;

pub type Lock<T> = RwLock<Box<T>>;

pub trait InterruptContext {
    fn restore(&self) -> !;
}

pub enum InterruptHandlerAction<K> {
    Procedure(fn()),
    Stub(fn(&dyn InterruptContext)),
    NeedPlatform(fn(&dyn InterruptContext, &mut Platform<K>)),
    NeedKernel(fn(&dyn InterruptContext, &mut K)),
    NeedPlatformAndKernel(fn(&dyn InterruptContext, &mut Platform<K>, &mut K)),
}

pub struct InterruptHandler<K> {
    pub name: String,
    pub action: InterruptHandlerAction<K>,
}

pub trait Core<K> {
    fn is_in_use(&self) -> bool;
    fn start(&mut self) -> MsgResult<()>;
    fn is_boot_processor(&self) -> bool;
    fn frequency_hz(&self) -> u64;
    fn manufacturer(&self) -> &str;
    fn model(&self) -> &str;

    fn interrupt_handlers(&self) -> &[(usize, InterruptHandler<K>)];

    fn register_interrupt_handler(
        &mut self,
        int: usize,
        handler: InterruptHandler<K>,
        prefer_fast: bool,
    ) -> MsgResult<()>;

    fn unregister_interrupt_handler(
        &mut self,
        int: usize,
    ) -> MsgResult<InterruptHandler<K>>;

    fn disable_interrupts(&mut self);
    fn enable_interrupts(&mut self);
}

pub enum Frame {
    /// Target any free frame in RAM
    NormalAnywhere,
    /// Target a specific frame in RAM
    Normal(u64),
    /// Device memory isn't cached and its accesses
    /// are unoptimized. It should be used for
    /// memory-mapped IO & configuration.
    Device(u64),
}

pub struct MappingOptions {
    pub frame: Frame,
    pub writeable: bool,
    /// set to `true` if this frame will
    /// contain code
    pub executable: bool,
    /// if set to `true`, userspace won't be
    /// able to read or write these bytes.
    pub restricted: bool,
}

pub trait Memory: Deref + DerefMut {}

pub type Mapping = Box<dyn Memory<Target = [u8]>>;

pub trait MemoryManager {
    fn frame_size(&self) -> u64;
    fn map_memory(&mut self, page: u64, count: usize, options: MappingOptions) -> MsgResult<Mapping>;
}

pub trait PowerManager { /* todo */ }

pub trait Logger {
    fn log(&mut self, message: &str);
}

pub trait Rng { /* todo */ }

pub trait Driven {
    fn driver(&self) -> &str;
    fn manufacturer(&self) -> &str;
    fn model(&self) -> &str;
}

pub struct PciDeviceInfo {
    pub vendor_id: u16,
    pub device_id: u16,
    pub base_class_code: u8,
    pub sub_class_code: u8,
    pub programming_interface: u8,
    pub grabbed: bool,
}

pub trait PciDevice { /* todo */ }

pub trait PciController: Driven {
    fn devices(&self) -> &[PciDeviceInfo];
    fn grab(&mut self, index: usize) -> MsgResult<Box<dyn PciDevice>>;
    fn ungrab(&mut self, device: Box<dyn PciDevice>) -> MsgResult<()>;
}

pub struct UsbDeviceInfo {
    pub vendor_id: u16,
    pub device_id: u16,
    pub class: u8,
    pub sub_class: u8,
    pub protocol: u8,
    pub release: u16,
    pub manufacturer: String,
    pub product: String,
    pub serial_number: String,
    pub grabbed: bool,
}

pub trait UsbDevice { /* todo */ }

pub trait UsbController: Driven {
    fn devices(&self) -> &[UsbDeviceInfo];
    fn grab(&mut self, index: usize) -> MsgResult<Box<dyn UsbDevice>>;
    fn ungrab(&mut self, device: Box<dyn UsbDevice>) -> MsgResult<()>;
}

pub trait NicController: Driven { /* todo */ }

pub trait I2cController: Driven { /* todo */ }

pub trait I2sController: Driven { /* todo */ }

pub trait GpioController: Driven { /* todo */ }

pub trait StorageController: Driven { /* todo */ }

pub trait SoundCard: Driven { /* todo */ }

pub trait SerialPort: Driven {
    fn baud_rate(&self) -> u64;
    fn set_baud_rate(&mut self, baud_rate: u64) -> MsgResult<()>;
    fn ready_for_writing(&self) -> bool;
    fn has_incoming_data(&self) -> bool;
    fn write(&mut self, buffer: &[u8]) -> MsgResult<usize>;
    fn read(&mut self, buffer: &mut [u8]) -> MsgResult<usize>;
}

pub trait FrameBuffer: Driven {
    /// https://wiki.osdev.org/Double_Buffering
    fn supports_double_buffering(&self) -> bool;
    fn are_buffer_swaps_automatic(&self) -> bool;
    /// If the framebuffer supports double buffering,
    /// the back buffer will be returned, else you'll
    /// get direct access to the framebuffer.
    fn as_mut_slice(&mut self) -> &mut [u8];
    fn swap_buffers(&mut self) -> MsgResult<()>;
}

pub trait VideoInput: Driven { /* todo */ }

pub trait HidInput: Driven { /* todo */ }

pub trait Timer: Driven { /* todo */ }

pub struct Platform<K> {
    // set before captain starts:
    pub cores: Box<[Lock<dyn Core<K>>]>,
    pub memory_manager: Lock<dyn MemoryManager>,
    pub power_manager: Lock<dyn PowerManager>,
    pub logger: Lock<dyn Logger>,
    pub rng: Lock<dyn Rng>,

    // set before and after captain has started:
    pub pci_controllers: Vec<Lock<dyn PciController>>,
    pub usb_controllers: Vec<Lock<dyn UsbController>>,
    pub nic_controllers: Vec<Lock<dyn NicController>>,
    pub i2c_controllers: Vec<Lock<dyn I2cController>>,
    pub i2s_controllers: Vec<Lock<dyn I2sController>>,
    pub gpio_controllers: Vec<Lock<dyn GpioController>>,
    pub storage_controllers: Vec<Lock<dyn StorageController>>,
    pub sound_cards: Vec<Lock<dyn SoundCard>>,
    pub serial_ports: Vec<Lock<dyn SerialPort>>,
    pub framebuffers: Vec<Lock<dyn FrameBuffer>>,
    pub video_inputs: Vec<Lock<dyn VideoInput>>,
    pub hid_inputs: Vec<Lock<dyn HidInput>>,
    pub timers: Vec<Lock<dyn Timer>>,

    // additional fields the kernel might want to add:
    pub kernel: Lock<Option<K>>,
}

pub type DriverInit<K> = fn(&mut Platform<K>) -> MsgResult<()>;
