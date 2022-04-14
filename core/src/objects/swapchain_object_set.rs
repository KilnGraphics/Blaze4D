use std::any::Any;
use std::sync::Arc;
use ash::prelude::VkResult;
use ash::vk;
use ash::vk::{Fence, Image, ImageView, Semaphore, SwapchainKHR};
use crate::objects::{id, ObjectSet, SynchronizationGroup};
use crate::objects::id::{FenceId, ImageId, ImageViewId, ObjectSetId, SemaphoreId, SurfaceId, SwapchainId};
use crate::objects::image::{ImageDescription, ImageInfo, ImageViewDescription, ImageViewInfo};
use crate::objects::object_set::ObjectSetProvider;
use crate::objects::swapchain::SwapchainCreateDesc;
use crate::rosella::DeviceContext;

/// Swapchain object sets manage the creation of swapchains and have utilities for some common
/// objects needed for each image.
///
/// Derivative objects can be added in which case a object is created for each swapchain image.
/// ImageViews, binary Semaphores and Fences are currently supported as derivative objects.
///
/// The swapchain itself is created during the creation of the builder (this is necessary because
/// the builder needs to know the number of images that are in the swapchain). Just like with
/// resource object sets the derivative objects are only created during the
/// [`SwapchainObjectSetBuilder::build`] call.
///
/// # Examples
///
/// ```
/// # use rosella_rs::objects::swapchain::{SwapchainCreateDesc, SwapchainImageSpec};
/// # use rosella_rs::objects::{Format, ImageViewDescription, SwapchainObjectSetBuilder};
/// use ash::vk;
///
/// // Create a builder. The swapchain will be immediately created.
/// let mut builder = SwapchainObjectSetBuilder::new(
///     device,
///     surface_id,
///     SwapchainCreateDesc::make(
///         SwapchainImageSpec::make(
///             &Format::R8G8B8A8_SRGB,
///             vk::ColorSpaceKHR::SRGB_NONLINEAR,
///             1920, 1080
///         ),
///         1,
///         vk::ImageUsageFlags::SAMPLED,
///         vk::PresentModeKHR::MAILBOX
///     ),
///     None
/// ).unwrap();
///
/// // We can query information about the already created swapchain
/// let swapchain_id = builder.get_swapchain_id();
/// let image_count = builder.get_image_ids().len();
///
/// // Add a image view. One will be created for each image of the swapchain
/// let image_views = builder.add_views(ImageViewDescription::make_full(
///     vk::ImageViewType::TYPE_2D,
///     &Format::R8G8B8A8_SRGB,
///     vk::ImageAspectFlags::COLOR
/// ));
///
/// // Similar to image views one semaphore will be created for each swapchain image
/// let semaphores = builder.add_binary_semaphores();
///
/// // During the build call all derivative objects will be created.
/// let object_set = builder.build().unwrap();
///
/// // Now we can access the objects and swapchain
/// let swapchain = unsafe { object_set.get_swapchain_handle(swapchain_id) };
/// for view in image_views.iter() {
///     unsafe { object_set.get_image_view_handle(*view) };
/// }
///
/// // The swapchain and derivative objects will be destroyed when the object set is dropped. The
/// // object set type uses Arc internally so it can be cloned and the objects will only be dropped
/// // when all references have been dropped.
/// ```
pub struct SwapchainObjectSetBuilder {
    device: DeviceContext,
    set_id: ObjectSetId,
    surface: SurfaceId,
    swapchain: vk::SwapchainKHR,
    images: Box<[SwapchainImage]>,
    image_desc: ImageDescription,
    derivatives: Vec<DerivativeData>,
}

impl SwapchainObjectSetBuilder {
    /// Creates a new swapchain object set builder.
    ///
    /// The swapchain will be immediately created. If a synchronization group is specified it will
    /// be used for all images. Otherwise a new synchronization group will be created for each
    /// individual image.
    pub fn new(device: DeviceContext, surface_id: SurfaceId, desc: SwapchainCreateDesc, synchronization_group: Option<SynchronizationGroup>) -> VkResult<Self> {
        let swapchain_fn = device.get_extension::<ash::extensions::khr::Swapchain>().unwrap();

        let surface = device.get_surface(surface_id).unwrap();
        let mut swapchain_info = surface.lock_swapchain_info();

        let old_swapchain = swapchain_info.get_current_handle().unwrap_or(SwapchainKHR::null());

        let create_info = vk::SwapchainCreateInfoKHR::builder()
            .surface(surface.get_handle())
            .min_image_count(desc.min_image_count)
            .image_format(desc.image_spec.format.get_format())
            .image_color_space(desc.image_spec.color_space)
            .image_extent(desc.image_spec.extent)
            .image_array_layers(desc.image_spec.array_layers)
            .image_usage(desc.usage)
            .image_sharing_mode(vk::SharingMode::EXCLUSIVE)
            .pre_transform(desc.pre_transform)
            .composite_alpha(desc.composite_alpha)
            .present_mode(desc.present_mode)
            .clipped(desc.clipped)
            .old_swapchain(old_swapchain);

        let new_swapchain = unsafe {
            swapchain_fn.create_swapchain(&create_info, None)
        }?;

        swapchain_info.set_swapchain(new_swapchain);

        let images = unsafe {
            swapchain_fn.get_swapchain_images(new_swapchain)

        }.map_err(|err| {
            // If there was an error destroy the swapchain and clear the surface swapchain info
            unsafe {
                swapchain_fn.destroy_swapchain(new_swapchain, None);
            }
            swapchain_info.clear();

            err
        })?;

        // Need to keep this alive until we are done with all operations that could fail
        drop(swapchain_info);

        let image_desc = ImageDescription {
            spec: desc.image_spec.as_image_spec(),
            usage_flags: desc.usage,
        };

        let images : Box<_> = images.into_iter().map(|image| {
            let group = match &synchronization_group {
                None => SynchronizationGroup::new(device.clone()),
                Some(group) => group.clone(),
            };

            SwapchainImage {
                info: Arc::new(ImageInfo::new(image_desc, group)),
                handle: image,
            }
        }).collect();

        // After this point errors are handled by the drop function of the SwapchainObjectSetBuilder
        Ok(Self {
            device,
            set_id: ObjectSetId::new(),
            surface: surface_id,
            swapchain: new_swapchain,
            images,
            image_desc,
            derivatives: Vec::new(),
        })
    }

    pub fn get_image_description(&self) -> &ImageDescription {
        &self.image_desc
    }

    pub fn get_swapchain_id(&self) -> SwapchainId {
        SwapchainId::new(self.set_id, 0)
    }

    pub fn get_image_ids(&self) -> Box<[ImageId]> {
        (0..self.images.len()).map(|index| ImageId::new(self.set_id, index as u16)).collect()
    }

    fn get_next_index(&self) -> u16 {
        let index = self.derivatives.len();
        if index > u16::MAX as usize {
            panic!("Too many objects in object set");
        }
        index as u16
    }

    /// Adds a set of image views for each image of the swapchain
    pub fn add_views(&mut self, desc: ImageViewDescription) -> Box<[ImageViewId]> {
        self.derivatives.reserve(self.images.len());
        let mut ids = Vec::with_capacity(self.images.len());

        for (index, image) in self.images.as_ref().iter().enumerate() {
            ids.push(ImageViewId::new(self.set_id, self.get_next_index()));

            let image_id = ImageId::new(self.set_id, index as u16);
            self.derivatives.push(DerivativeData::make_image_view(desc, image_id, image.info.clone()));
        }

        ids.into_boxed_slice()
    }

    /// Adds a set of binary semaphores for each image of the swapchain
    pub fn add_binary_semaphores(&mut self) -> Box<[SemaphoreId]> {
        self.derivatives.reserve(self.images.len());
        let mut ids = Vec::with_capacity(self.images.len());

        for _ in self.images.as_ref() {
            ids.push(SemaphoreId::new(self.set_id, self.get_next_index()));
            self.derivatives.push(DerivativeData::make_binary_semaphore())
        }

        ids.into_boxed_slice()
    }

    /// Adds a set of fences for each image of the swapchain
    pub fn add_fences(&mut self) -> Box<[FenceId]> {
        self.derivatives.reserve(self.images.len());
        let mut ids = Vec::with_capacity(self.images.len());

        for _ in self.images.as_ref() {
            ids.push(FenceId::new(self.set_id, self.get_next_index()));
            self.derivatives.push(DerivativeData::make_fence())
        }

        ids.into_boxed_slice()
    }

    fn create(&mut self) -> Result<(), vk::Result> {
        for derivative in &mut self.derivatives {
            derivative.create(&self.device, &self.images)?;
        }

        Ok(())
    }

    fn destroy(&mut self) {
        for derivative in &mut self.derivatives {
            derivative.destroy(&self.device);
        }
    }

    pub fn build(mut self) -> Result<ObjectSet, vk::Result> {
        if let Err(err) = self.create() {
            self.destroy();
            return Err(err);
        }

        // This is beyond ugly but necessary since we implement drop
        Ok(ObjectSet::new(SwapchainObjectSet {
            device: self.device.clone(),
            set_id: self.set_id,
            surface: self.surface,
            swapchain: std::mem::replace(&mut self.swapchain, vk::SwapchainKHR::null()),
            images: std::mem::replace(&mut self.images, Box::new([])),
            derivatives: std::mem::replace(&mut self.derivatives, Vec::new()).into_boxed_slice(),
        }))
    }
}

impl Drop for SwapchainObjectSetBuilder {
    fn drop(&mut self) {
        if self.swapchain != vk::SwapchainKHR::null() {
            let swapchain_fn = self.device.get_extension::<ash::extensions::khr::Swapchain>().unwrap();

            let surface = self.device.get_surface(self.surface).unwrap();
            let mut swapchain_info = surface.lock_swapchain_info();

            unsafe {
                swapchain_fn.destroy_swapchain(self.swapchain, None)
            };

            if swapchain_info.get_current_handle() == Some(self.swapchain) {
                swapchain_info.clear();
            }
            self.swapchain = vk::SwapchainKHR::null();
        }
    }
}

struct ImageViewData {
    info: Box<ImageViewInfo>,
    handle: vk::ImageView,
}

impl ImageViewData {
    fn new(desc: ImageViewDescription, image_id: id::ImageId, image_info: Arc<ImageInfo>) -> Self {
        Self {
            info: Box::new(ImageViewInfo::new(desc, image_id, image_info)),
            handle: vk::ImageView::null(),
        }
    }

    fn create(&mut self, device: &DeviceContext, images: &Box<[SwapchainImage]>) -> Result<(), vk::Result> {
        if self.handle == vk::ImageView::null() {
            let index = self.info.get_source_image_id().get_index() as usize;

            let description = self.info.get_description();

            let info = vk::ImageViewCreateInfo::builder()
                .image(images.get(index).unwrap().handle)
                .view_type(description.view_type)
                .format(description.format.get_format())
                .components(description.components)
                .subresource_range(description.subresource_range.as_vk_subresource_range());

            self.handle = unsafe {
                device.vk().create_image_view(&info, None)
            }?;
        }

        Ok(())
    }

    fn destroy(&mut self, device: &DeviceContext) {
        if self.handle != vk::ImageView::null() {
            unsafe { device.vk().destroy_image_view(self.handle, None) };
            self.handle = vk::ImageView::null();
        }
    }
}

struct BinarySemaphoreData {
    handle: vk::Semaphore,
}

impl BinarySemaphoreData {
    fn new() -> Self {
        Self {
            handle: vk::Semaphore::null(),
        }
    }

    fn create(&mut self, device: &DeviceContext) -> Result<(), vk::Result> {
        if self.handle == vk::Semaphore::null() {
            let info = vk::SemaphoreCreateInfo::builder();

            let handle = unsafe {
                device.vk().create_semaphore(&info, None)
            }?;
            self.handle = handle;
        }

        Ok(())
    }

    fn destroy(&mut self, device: &DeviceContext) {
        if self.handle != vk::Semaphore::null() {
            unsafe { device.vk().destroy_semaphore(self.handle, None) };
            self.handle = vk::Semaphore::null();
        }
    }
}

struct FenceData {
    handle: vk::Fence,
}

impl FenceData {
    fn new() -> Self {
        Self {
            handle: vk::Fence::null(),
        }
    }

    fn create(&mut self, device: &DeviceContext) -> Result<(), vk::Result> {
        if self.handle == vk::Fence::null() {
            let info = vk::FenceCreateInfo::builder();

            let handle = unsafe {
                device.vk().create_fence(&info, None)
            }?;
            self.handle = handle;
        }

        Ok(())
    }

    fn destroy(&mut self, device: &DeviceContext) {
        if self.handle != vk::Fence::null() {
            unsafe { device.vk().destroy_fence(self.handle, None) };
            self.handle = vk::Fence::null();
        }
    }
}

enum DerivativeData {
    ImageView(ImageViewData),
    BinarySemaphore(BinarySemaphoreData),
    Fence(FenceData),
}

impl DerivativeData {
    fn make_image_view(desc: ImageViewDescription, image_id: id::ImageId, image_info: Arc<ImageInfo>) -> Self {
        Self::ImageView(ImageViewData::new(desc, image_id, image_info))
    }

    fn make_binary_semaphore() -> Self {
        Self::BinarySemaphore(BinarySemaphoreData::new())
    }

    fn make_fence() -> Self {
        Self::Fence(FenceData::new())
    }

    fn create(&mut self, device: &DeviceContext, images: &Box<[SwapchainImage]>) -> Result<(), vk::Result> {
        match self {
            DerivativeData::ImageView(data) => data.create(device, images),
            DerivativeData::BinarySemaphore(data) => data.create(device),
            DerivativeData::Fence(data) => data.create(device)
        }
    }

    fn destroy(&mut self, device: &DeviceContext) {
        match self {
            DerivativeData::ImageView(data) => data.destroy(device),
            DerivativeData::BinarySemaphore(data) => data.destroy(device),
            DerivativeData::Fence(data) => data.destroy(device),
        }
    }
}

struct SwapchainImage {
    info: Arc<ImageInfo>,
    handle: vk::Image,
}

struct SwapchainObjectSet {
    device: DeviceContext,
    set_id: ObjectSetId,
    surface: SurfaceId,
    swapchain: vk::SwapchainKHR,
    images: Box<[SwapchainImage]>,
    derivatives: Box<[DerivativeData]>,
}

impl SwapchainObjectSet {

}

impl ObjectSetProvider for SwapchainObjectSet {
    fn get_id(&self) -> ObjectSetId {
        self.set_id
    }

    unsafe fn get_image_handle(&self, id: ImageId) -> Image {
        if id.get_set_id() != self.set_id {
            panic!("Image belongs to different object set");
        }

        let index = id.get_index() as usize;
        self.images.get(index).unwrap().handle
    }

    fn get_image_info(&self, id: ImageId) -> &Arc<ImageInfo> {
        if id.get_set_id() != self.set_id {
            panic!("Image belongs to different object set");
        }

        let index = id.get_index() as usize;
        &self.images.get(index).unwrap().info
    }

    unsafe fn get_image_view_handle(&self, id: ImageViewId) -> ImageView {
        if id.get_set_id() != self.set_id {
            panic!("ImageView belongs to different object set");
        }

        let index = id.get_index() as usize;
        match self.derivatives.get(index).unwrap() {
            DerivativeData::ImageView(data) => data.handle,
            _ => panic!("Id does not map to image view"),
        }
    }

    fn get_image_view_info(&self, id: ImageViewId) -> &ImageViewInfo {
        if id.get_set_id() != self.set_id {
            panic!("ImageView belongs to different object set");
        }

        let index = id.get_index() as usize;
        match self.derivatives.get(index).unwrap() {
            DerivativeData::ImageView(data) => data.info.as_ref(),
            _ => panic!("Id does not map to image view"),
        }
    }

    unsafe fn get_swapchain_handle(&self, id: SwapchainId) -> SwapchainKHR {
        if id != SwapchainId::new(self.set_id, 0) {
            panic!("Invalid SwapchainId")
        }

        self.swapchain
    }

    unsafe fn get_semaphore_handle(&self, id: SemaphoreId) -> Semaphore {
        if id.get_set_id() != self.set_id {
            panic!("Semaphore belongs to different object set");
        }

        let index = id.get_index() as usize;
        match self.derivatives.get(index).unwrap() {
            DerivativeData::BinarySemaphore(data) => data.handle,
            _ => panic!("Id does not map to semaphore"),
        }
    }

    unsafe fn get_fence_handle(&self, id: FenceId) -> Fence {
        if id.get_set_id() != self.set_id {
            panic!("Fence belongs to different object set");
        }

        let index = id.get_index() as usize;
        match self.derivatives.get(index).unwrap() {
            DerivativeData::Fence(data) => data.handle,
            _ => panic!("Id does not map to fence"),
        }
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Drop for SwapchainObjectSet {
    fn drop(&mut self) {
        for derivative in self.derivatives.as_mut() {
            derivative.destroy(&self.device);
        }

        if self.swapchain != vk::SwapchainKHR::null() {
            let swapchain_fn = self.device.get_extension::<ash::extensions::khr::Swapchain>().unwrap();

            let surface = self.device.get_surface(self.surface).unwrap();
            let mut swapchain_info = surface.lock_swapchain_info();

            unsafe {
                swapchain_fn.destroy_swapchain(self.swapchain, None)
            };

            if swapchain_info.get_current_handle() == Some(self.swapchain) {
                swapchain_info.clear();
            }
            self.swapchain = vk::SwapchainKHR::null();
        }
    }
}

#[cfg(test)]
mod tests {
    // TODO how on earth do we test this???
}