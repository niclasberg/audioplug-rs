use std::cell::RefCell;
use std::fmt::Write;
use std::marker::PhantomData;
use std::ops::Deref;

use bytemuck::{bytes_of, try_from_bytes, try_from_bytes_mut, Pod, Zeroable};
use windows::Win32::Foundation::{E_INVALIDARG, E_NOTIMPL, E_POINTER};
use windows::Win32::Graphics::Direct2D;
use windows::{core::Result, Win32::Foundation::RECT};
use windows_core::{implement, ComObjectInterface, IUnknown, Interface, HSTRING, PCWSTR};

use crate::core::{Color, Vec2f, Vec3f, Vec4f};

use super::renderer::{DeviceResource, RendererGeneration};

pub trait CustomEffect: Pod + 'static + Default {
    const NAME: &'static str;
    const EFFECT_GUID: windows_core::GUID;
    const SHADER_GUID: windows_core::GUID;
    fn shader_bytes() -> &'static [u8];
    fn extent(&self) -> RECT;
}

#[implement(
    Direct2D::ID2D1EffectImpl,
    Direct2D::ID2D1DrawTransform,
    Direct2D::ID2D1Transform,
    Direct2D::ID2D1TransformNode
)]
struct CustomEffectImpl<T>
where
    T: CustomEffect,
{
    draw_info: RefCell<Option<Direct2D::ID2D1DrawInfo>>,
    consts: RefCell<T>,
}

impl<T: CustomEffect> Direct2D::ID2D1EffectImpl_Impl for CustomEffectImpl_Impl<T> {
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

    fn PrepareForRender(
        &self,
        _change_type: Direct2D::D2D1_CHANGE_TYPE,
    ) -> windows_core::Result<()> {
        unsafe {
            let consts = self.consts.borrow();

            self.draw_info
                .borrow()
                .as_ref()
                .unwrap()
                .SetPixelShaderConstantBuffer(bytes_of(consts.deref()))
        }?;
        Ok(())
    }

    fn SetGraph(
        &self,
        _transform_graph: windows_core::Ref<'_, Direct2D::ID2D1TransformGraph>,
    ) -> windows_core::Result<()> {
        // SetGraph is only called when the number of inputs changes. This never happens as we publish this effect
        // as a single input effect.
        Err(windows_core::Error::from_hresult(E_NOTIMPL))
    }
}

impl<T: CustomEffect> Direct2D::ID2D1DrawTransform_Impl for CustomEffectImpl_Impl<T> {
    fn SetDrawInfo(
        &self,
        draw_info: windows_core::Ref<'_, Direct2D::ID2D1DrawInfo>,
    ) -> windows_core::Result<()> {
        let draw_info = draw_info.ok()?;
        self.draw_info.replace(Some(draw_info.clone()));
        unsafe { draw_info.SetPixelShader(&T::SHADER_GUID, Direct2D::D2D1_PIXEL_OPTIONS_NONE) }
    }
}

impl<T: CustomEffect> Direct2D::ID2D1Transform_Impl for CustomEffectImpl_Impl<T> {
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
        _input_rects: *const RECT,
        _input_opaque_sub_rects: *const RECT,
        input_rect_count: u32,
        output_rect: *mut RECT,
        output_opaque_sub_rect: *mut RECT,
    ) -> windows_core::Result<()> {
        if input_rect_count != 0 {
            Err(windows_core::Error::from_hresult(E_INVALIDARG))
        } else {
            let extent = self.consts.borrow().extent();
            unsafe {
                *(output_rect as *mut _) = extent;
                *(output_opaque_sub_rect as *mut _) = extent;
            }
            Ok(())
        }
    }

    fn MapInvalidRect(
        &self,
        _input_index: u32,
        _invalid_inputrect: &RECT,
    ) -> windows_core::Result<RECT> {
        // No inputs...
        Err(windows_core::Error::from_hresult(E_INVALIDARG))
    }
}

impl<T: CustomEffect> Direct2D::ID2D1TransformNode_Impl for CustomEffectImpl_Impl<T> {
    fn GetInputCount(&self) -> u32 {
        0
    }
}

#[derive(Clone)]
pub struct EffectWrapper<T> {
    effect: Direct2D::ID2D1Effect,
    _phantom: PhantomData<T>,
}

impl<T: CustomEffect> EffectWrapper<T> {
    pub fn register(factory: &Direct2D::ID2D1Factory1) -> windows_core::Result<()> {
        let dummy_prop_name: HSTRING = "Constants".into();
        let bindings = [Direct2D::D2D1_PROPERTY_BINDING {
            propertyName: PCWSTR(dummy_prop_name.as_ptr()),
            setFunction: Some(Self::constants_setter),
            getFunction: Some(Self::constants_getter),
        }];

        let mut xml = String::new();
        write!(
            &mut xml,
            "<?xml version='1.0'?>
            <Effect>
                <!-- System Properties -->
                <Property name='DisplayName' type='string' value='{}'/>
                <Property name='Author' type='string' value='Audioplug-rs'/>
                <Property name='Category' type='string' value='Stylize'/>
                <Property name='Description' type='string' value='Audioplug custom effect'/>
                <Inputs>
                </Inputs>

                <!-- Custom Properties go here -->
                <Property name='Constants' type='blob'>
                    <Property name='DisplayName' type='string' value='Constants' />
                </Property>
            </Effect>",
            T::NAME
        )
        .expect("Unable to format CustomEffect xml");

        let xml_hstring: HSTRING = xml.into();
        unsafe {
            factory.RegisterEffectFromString(
                &T::EFFECT_GUID,
                PCWSTR(xml_hstring.as_ptr()),
                Some(bindings.as_slice()),
                Some(Self::create),
            )
        }
    }

    /// Create a new instance of the effect. The Self::register method must have been called beforehand
    pub fn new(device_context: &Direct2D::ID2D1DeviceContext) -> Result<Self> {
        let effect = unsafe { device_context.CreateEffect(&T::EFFECT_GUID) }?;
        Ok(Self {
            effect,
            _phantom: PhantomData,
        })
    }

    pub fn effect(&self) -> &Direct2D::ID2D1Effect {
        &self.effect
    }

    pub fn set_constants(&self, value: T) {
        unsafe {
            self.effect
                .SetValue(0, Direct2D::D2D1_PROPERTY_TYPE_BLOB, bytes_of(&value))
        }
        .unwrap();
    }

    unsafe extern "system" fn constants_setter(
        effect: windows_core::Ref<'_, windows_core::IUnknown>,
        data: *const u8,
        datasize: u32,
    ) -> windows_core::HRESULT {
        let Some(effect) = effect.as_ref() else {
            return E_POINTER;
        };

        let eff = match effect.cast_object_ref::<CustomEffectImpl<T>>() {
            Ok(eff) => eff,
            Err(err) => return err.into(),
        };

        let slice = unsafe { std::slice::from_raw_parts(data, datasize as _) };
        let Ok(value) = try_from_bytes(slice) else {
            return E_INVALIDARG;
        };
        let mut consts = eff.consts.borrow_mut();
        *consts = *value;

        windows_core::HRESULT(0)
    }

    unsafe extern "system" fn constants_getter(
        effect: windows_core::Ref<'_, windows_core::IUnknown>,
        data: *mut u8,
        data_size: u32,
        actual_size: *mut u32,
    ) -> windows_core::HRESULT {
        let Some(effect) = effect.as_ref() else {
            return E_POINTER;
        };

        *actual_size = std::mem::size_of::<T>() as u32;

        // Size query
        if data_size == 0 {
            return windows_core::HRESULT(0);
        }

        let eff = match effect.cast_object_ref::<CustomEffectImpl<T>>() {
            Ok(eff) => eff,
            Err(err) => return err.into(),
        };

        if actual_size.is_null() {
            return E_POINTER;
        }

        let slice = unsafe { std::slice::from_raw_parts_mut(data, data_size as _) };
        let Ok(value) = try_from_bytes_mut(slice) else {
            return E_INVALIDARG;
        };

        let consts = eff.consts.borrow();
        *value = *consts;

        windows_core::HRESULT(0)
    }

    unsafe extern "system" fn create(
        effect_impl: windows_core::OutRef<'_, windows_core::IUnknown>,
    ) -> windows_core::HRESULT {
        let this = CustomEffectImpl {
            draw_info: RefCell::new(None),
            consts: RefCell::new(T::default()),
        };
        let com_object: IUnknown = this.into();

        match effect_impl.write(Some(com_object)) {
            Ok(_) => windows_core::HRESULT::default(),
            Err(e) => e.into(),
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RectShadow {
    pub shadow_color: Vec4f,
    pub size: Vec2f,
    pub shadow_offset: Vec2f,
    pub shadow_radius: f32,
    pub padding: Vec3f,
}

unsafe impl Zeroable for RectShadow {}
unsafe impl Pod for RectShadow {}

impl CustomEffect for RectShadow {
    const NAME: &'static str = "RectShadowEffect";
    const EFFECT_GUID: windows_core::GUID = windows_core::GUID::from_values(
        0x9f489c12,
        0x9434,
        0x4b26,
        [0x8c, 0xb1, 0x4c, 0xfe, 0xf5, 0xe5, 0x2c, 0xeb],
    );

    const SHADER_GUID: windows_core::GUID = windows_core::GUID::from_values(
        0xfd9496d9,
        0x233d,
        0x43c4,
        [0x94, 0x24, 0xef, 0x54, 0x26, 0xe6, 0xb4, 0x69],
    );

    fn shader_bytes() -> &'static [u8] {
        std::include_bytes!("shaders/rect_shadow.cso")
    }

    fn extent(&self) -> RECT {
        rect_shadow_extent(self.size, self.shadow_offset, self.shadow_radius)
    }
}

fn rect_shadow_extent(size: Vec2f, offset: Vec2f, radius: f32) -> RECT {
    let top_left = (Vec2f::ZERO.min(offset) - Vec2f::splat(radius)).floor();
    let bottom_right = (size + Vec2f::ZERO.max(offset) + Vec2f::splat(radius)).ceil();
    RECT {
        left: top_left.x as i32,
        top: top_left.y as i32,
        right: bottom_right.x as i32,
        bottom: bottom_right.y as i32,
    }
}

const DEFAULT_SHADOW_COLOR: Vec4f = Vec4f {
    x: 0.0,
    y: 0.0,
    z: 0.0,
    w: 0.3,
};

impl Default for RectShadow {
    fn default() -> Self {
        Self {
            size: Default::default(),
            shadow_radius: Default::default(),
            shadow_offset: Default::default(),
            shadow_color: DEFAULT_SHADOW_COLOR,
            padding: Default::default(),
        }
    }
}

pub type RectShadowEffect = EffectWrapper<RectShadow>;

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RoundedRectShadow {
    pub shadow_color: Vec4f,
    pub size: Vec2f,
    pub offset: Vec2f,
    pub corner_radius: f32,
    pub shadow_radius: f32,
    pub padding: Vec2f,
}

unsafe impl Zeroable for RoundedRectShadow {}
unsafe impl Pod for RoundedRectShadow {}

impl CustomEffect for RoundedRectShadow {
    const NAME: &'static str = "RoundedRectShadowEffect";
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

    fn extent(&self) -> RECT {
        rect_shadow_extent(self.size, self.offset, self.shadow_radius)
    }
}

impl Default for RoundedRectShadow {
    fn default() -> Self {
        Self {
            shadow_color: DEFAULT_SHADOW_COLOR,
            size: Vec2f { x: 1.0, y: 1.0 },
            offset: Vec2f { x: 0.0, y: 0.0 },
            corner_radius: 0.0,
            shadow_radius: 0.0,
            padding: Default::default(),
        }
    }
}

pub type RoundedRectShadowEffect = EffectWrapper<RoundedRectShadow>;

pub struct Image {
    bitmap: DeviceResource<Direct2D::ID2D1Bitmap>,
}

pub(super) struct GaussianBlur {
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
