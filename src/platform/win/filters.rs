use std::cell::RefCell;

use bytemuck::{bytes_of, NoUninit};
use windows::Win32::Foundation::{E_INVALIDARG, E_NOTIMPL};
use windows::Win32::Graphics::Direct2D;
use windows::{core::Result, Win32::Foundation::RECT};
use windows_core::{implement, w, ComObjectInterface, IUnknown, InterfaceRef, PCWSTR};

use crate::core::Color;

use super::renderer::{DeviceResource, RendererGeneration};

#[repr(C)]
#[derive(Copy, Clone)]
struct EffectConstants {
    offset: windows_numerics::Vector2,
    std_dev: f32,
}

unsafe impl NoUninit for EffectConstants {}

pub trait PropertyGetter {}

pub enum PropertyType {
    Float,
}

pub struct PropertyBinding {
    name: PCWSTR,
    setter: Direct2D::PD2D1_PROPERTY_SET_FUNCTION,
    getter: Direct2D::PD2D1_PROPERTY_GET_FUNCTION,
    xml: PCWSTR,
}

macro_rules! property_binding {
    ($parent: ty, $name: expr) => {
        PropertyBinding {
            name: w!($name),
            /*setter: Some(
                move |effect: windows_core::Ref<windows_core::IUnknown>,
                      data: *const u8,
                      datasize: u32|
                      -> windows_core::HRESULT { windows_core::HRESULT(0) },
            ),
            getter: Some(
                |effect: windows_core::Ref<'_, windows_core::IUnknown>,
                 data: *mut u8,
                 datasize: u32,
                 actualsize: *mut u32|
                 -> windows_core::HRESULT { windows_core::HRESULT(0) },
            ),*/
            setter: None,
            getter: None,
            xml: w!(""),
        }
    };
}

pub trait CustomEffect: NoUninit + 'static + Default {
    const NAME: PCWSTR;
    const PROPERTIES: &[PropertyBinding];
    const EFFECT_GUID: windows_core::GUID;
    const SHADER_GUID: windows_core::GUID;
    fn shader_bytes() -> &'static [u8];
}

#[implement(
    Direct2D::ID2D1EffectImpl,
    Direct2D::ID2D1DrawTransform,
    Direct2D::ID2D1Transform,
    Direct2D::ID2D1TransformNode
)]
pub struct EffectWrapper<T>
where
    T: CustomEffect,
{
    draw_info: RefCell<Option<Direct2D::ID2D1DrawInfo>>,
    consts: T,
}

impl<T: CustomEffect> EffectWrapper<T> {
    unsafe extern "system" fn create(
        effect_impl: windows_core::OutRef<'_, windows_core::IUnknown>,
    ) -> windows_core::HRESULT {
        let this = Self {
            draw_info: RefCell::new(None),
            consts: T::default(),
        }
        .into_outer();
        let a: InterfaceRef<IUnknown> = this.as_interface_ref();
        let result = effect_impl.write(Some(a.to_owned()));

        match result {
            Ok(_) => windows_core::HRESULT::default(),
            Err(e) => e.into(),
        }
    }

    pub fn register(factory: &Direct2D::ID2D1Factory1) -> windows_core::Result<()> {
        let bindings: Vec<_> = T::PROPERTIES
            .iter()
            .map(|prop| Direct2D::D2D1_PROPERTY_BINDING {
                propertyName: prop.name,
                setFunction: prop.setter,
                getFunction: prop.getter,
            })
            .collect();
        let property_xml = w!("");
        unsafe {
            factory.RegisterEffectFromString(
                &T::EFFECT_GUID,
                property_xml,
                Some(bindings.as_slice()),
                Some(Self::create),
            )
        }
    }
}

impl<T: CustomEffect> Direct2D::ID2D1EffectImpl_Impl for EffectWrapper_Impl<T> {
    fn Initialize(
        &self,
        context: windows_core::Ref<'_, Direct2D::ID2D1EffectContext>,
        transform_graph: windows_core::Ref<'_, Direct2D::ID2D1TransformGraph>,
    ) -> windows_core::Result<()> {
        let cx = context.ok()?;
        let graph = transform_graph.ok()?;
        unsafe { cx.LoadPixelShader(&T::SHADER_GUID, T::shader_bytes()) }?;
        unsafe { graph.SetSingleTransformNode(self.as_interface_ref()) }?;
        Ok(())
    }

    fn PrepareForRender(&self, changetype: Direct2D::D2D1_CHANGE_TYPE) -> windows_core::Result<()> {
        unsafe {
            self.draw_info
                .borrow()
                .as_ref()
                .unwrap()
                .SetPixelShaderConstantBuffer(bytes_of(&self.consts))
        }?;
        Ok(())
    }

    fn SetGraph(
        &self,
        _transformgraph: windows_core::Ref<'_, Direct2D::ID2D1TransformGraph>,
    ) -> windows_core::Result<()> {
        // SetGraph is only called when the number of inputs changes. This never happens as we publish this effect
        // as a single input effect.
        Err(windows_core::Error::from_hresult(E_NOTIMPL))
    }
}

impl<T: CustomEffect> Direct2D::ID2D1DrawTransform_Impl for EffectWrapper_Impl<T> {
    fn SetDrawInfo(
        &self,
        draw_info: windows_core::Ref<'_, Direct2D::ID2D1DrawInfo>,
    ) -> windows_core::Result<()> {
        let draw_info = draw_info.ok()?;
        self.draw_info.replace(Some(draw_info.clone()));
        unsafe { draw_info.SetPixelShader(&T::SHADER_GUID, Direct2D::D2D1_PIXEL_OPTIONS_NONE) }
    }
}

impl<T: CustomEffect> Direct2D::ID2D1Transform_Impl for EffectWrapper_Impl<T> {
    fn MapOutputRectToInputRects(
        &self,
        _output_rect: *const RECT,
        _input_rects: *mut RECT,
        input_rect_count: u32,
    ) -> windows_core::Result<()> {
        // This effect has no inputs.
        if input_rect_count != 0 {
            Err(windows_core::Error::from_hresult(E_INVALIDARG))
        } else {
            Ok(())
        }
    }

    fn MapInputRectsToOutputRect(
        &self,
        inputrects: *const RECT,
        inputopaquesubrects: *const RECT,
        inputrectcount: u32,
        outputrect: *mut RECT,
        outputopaquesubrect: *mut RECT,
    ) -> windows_core::Result<()> {
        todo!()
    }

    fn MapInvalidRect(
        &self,
        inputindex: u32,
        invalidinputrect: &RECT,
    ) -> windows_core::Result<RECT> {
        todo!()
    }
}

impl<T: CustomEffect> Direct2D::ID2D1TransformNode_Impl for EffectWrapper_Impl<T> {
    fn GetInputCount(&self) -> u32 {
        0
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
struct RoundedRectShadowEffectShader {
    shadow_color: windows_numerics::Vector4,
    size: windows_numerics::Vector2,
    offset: windows_numerics::Vector2,
    corner_radius: f32,
    shadow_radius: f32,
}

unsafe impl NoUninit for RoundedRectShadowEffectShader {}

impl CustomEffect for RoundedRectShadowEffectShader {
    const NAME: PCWSTR = w!("RoundedRectangleShadow");
    const PROPERTIES: &[PropertyBinding] = &[property_binding!(
        RoundedRectShadowEffectShader,
        "shadow_color"
    )];
    const EFFECT_GUID: windows_core::GUID = windows_core::GUID::from_values(
        0x792b02fc,
        0x1b12,
        0x4ee3,
        [0x88, 0xcd, 0xdc, 0x57, 0xd6, 0xb, 0x91, 0xa5],
    );
    const SHADER_GUID: windows_core::GUID = windows_core::GUID::from_values(
        0xda83698b,
        0x4b5a,
        0x4803,
        [0x82, 0x23, 0x0e, 0x86, 0x13, 0xe9, 0x3f, 0x17],
    );

    fn shader_bytes() -> &'static [u8] {
        std::include_bytes!("shaders/rounded_rect_shadow.cso")
    }
}

impl RoundedRectShadowEffectShader {}

impl Default for RoundedRectShadowEffectShader {
    fn default() -> Self {
        Self {
            shadow_color: windows_numerics::Vector4 {
                X: 0.0,
                Y: 0.0,
                Z: 0.0,
                W: 0.3,
            },
            size: windows_numerics::Vector2 { X: 1.0, Y: 1.0 },
            offset: windows_numerics::Vector2 { X: 0.0, Y: 0.0 },
            corner_radius: 0.0,
            shadow_radius: 0.0,
        }
    }
}

pub type RoundedRectShadow = EffectWrapper<RoundedRectShadowEffectShader>;
impl RoundedRectShadow {
    pub fn set_offset(&self) {}
}

pub struct Image {
    bitmap: DeviceResource<Direct2D::ID2D1Bitmap>,
}

pub struct GaussianBlur {
    filter: DeviceResource<Direct2D::ID2D1Effect>,
    radius: f64,
}

impl GaussianBlur {
    pub fn new(radius: f64) -> Self {
        Self {
            filter: DeviceResource::new(),
            radius,
        }
    }

    pub fn set_radius(&mut self, radius: f64) {
        self.radius = radius;
    }

    pub fn with_output(
        &self,
        render_target: &Direct2D::ID2D1DeviceContext,
        generation: RendererGeneration,
        f: impl FnOnce(&Direct2D::ID2D1DeviceContext, &Direct2D::ID2D1Image),
    ) -> Result<()> {
        let filter = self.filter.get_or_insert(generation, || unsafe {
            render_target.CreateEffect(&Direct2D::CLSID_D2D1GaussianBlur)
        })?;

        unsafe {
            set_effect_property_f32(
                &filter,
                Direct2D::D2D1_GAUSSIANBLUR_PROP_STANDARD_DEVIATION.0,
                self.radius as _,
            )
        }?;

        f(render_target, &unsafe { filter.GetOutput() }?);

        Ok(())
        //self.filter.get_or_insert(generation, || {})
    }
}

pub struct DropShadow {}

impl DropShadow {}

pub unsafe fn set_effect_property_f32(
    properties: &Direct2D::ID2D1Properties,
    index: i32,
    value: f32,
) -> Result<()> {
    properties.SetValue(
        index as u32,
        Direct2D::D2D1_PROPERTY_TYPE_FLOAT,
        value.to_ne_bytes().as_slice(),
    )
}

pub unsafe fn set_effect_property_u32(
    properties: &Direct2D::ID2D1Properties,
    index: i32,
    value: u32,
) -> Result<()> {
    properties.SetValue(
        index as u32,
        Direct2D::D2D1_PROPERTY_TYPE_UINT32,
        value.to_ne_bytes().as_slice(),
    )
}

pub fn set_effect_property_i32(
    properties: &Direct2D::ID2D1Properties,
    index: i32,
    value: i32,
) -> Result<()> {
    unsafe {
        properties.SetValue(
            index as u32,
            Direct2D::D2D1_PROPERTY_TYPE_INT32,
            value.to_ne_bytes().as_slice(),
        )
    }
}
