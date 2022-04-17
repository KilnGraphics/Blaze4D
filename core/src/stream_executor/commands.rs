use ash::vk;
use crate::objects::id::ImageViewId;

#[non_exhaustive]
#[derive(Clone)]
struct RenderingInfo {
    render_area: vk::Rect2D,
    layer_count: u32,
    color_attachments: Vec<u32>,
    depth_attachment: Option<RenderingAttachmentInfo>,
    stencil_attachment: Option<RenderingAttachmentInfo>,
}

#[non_exhaustive]
#[derive(Copy, Clone)]
struct RenderingAttachmentInfo {

}