use std::cell::RefCell;
use std::fmt::{Display, Write};
use std::ops::Deref;

use bytemuck::{bytes_of, try_from_bytes, try_from_bytes_mut, NoUninit};
use windows::Win32::Foundation::{E_INVALIDARG, E_NOTIMPL, E_POINTER};
use windows::Win32::Graphics::Direct2D;
use windows::{core::Result, Win32::Foundation::RECT};
use windows_core::{implement, w, ComObjectInterface, IUnknown, Interface, HSTRING, PCWSTR};

use crate::core::{Vec2f, Vec3f, Vec4f};

use super::renderer::{DeviceResource, RendererGeneration};

pub struct PropertyBinding {
    name: &'static str,
    setter: Direct2D::PD2D1_PROPERTY_SET_FUNCTION,
    getter: Direct2D::PD2D1_PROPERTY_GET_FUNCTION,
    type_name: &'static str,
}

impl PropertyBinding {
    fn xml_str(&self) -> String {
        format!(
            "<Property name='{}' type='{}'>
                <Property name='DisplayName' type='string' value='{}' />
            </Property>",
            self.name, self.type_name, self.name
        )
    }
}

macro_rules! property_binding {
    ($parent: ty, $name: expr, $path: ident, $kind: expr) => {{
        unsafe extern "system" fn setter(
            effect: windows_core::Ref<'_, windows_core::IUnknown>,
            data: *const u8,
            datasize: u32,
        ) -> windows_core::HRESULT {
            let Some(effect) = effect.as_ref() else {
                return E_POINTER;
            };

            let eff = match effect.cast_object_ref::<EffectWrapper<$parent>>() {
                Ok(eff) => eff,
                Err(err) => return err.into(),
            };

            let slice = unsafe { std::slice::from_raw_parts(data, datasize as _) };
            let Ok(value) = try_from_bytes(slice) else {
                return E_INVALIDARG;
            };

            let mut consts = eff.consts.borrow_mut();
            consts.$path = *value;

            windows_core::HRESULT(0)
        }

        unsafe extern "system" fn getter(
            effect: windows_core::Ref<'_, windows_core::IUnknown>,
            data: *mut u8,
            datasize: u32,
            actualsize: *mut u32,
        ) -> windows_core::HRESULT {
            let Some(effect) = effect.as_ref() else {
                return E_POINTER;
            };

            let eff = match effect.cast_object_ref::<EffectWrapper<$parent>>() {
                Ok(eff) => eff,
                Err(err) => return err.into(),
            };

            let slice = unsafe { std::slice::from_raw_parts_mut(data, datasize as _) };
            let Ok(value) = try_from_bytes_mut(slice) else {
                return E_INVALIDARG;
            };

            let consts = eff.consts.borrow();
            *value = consts.$path;
            *actualsize = std::mem::size_of_val(&consts.$path) as u32;

            windows_core::HRESULT(0)
        }

        PropertyBinding {
            name: $name,
            setter: Some(setter),
            getter: Some(getter),
            type_name: $kind,
        }
    }};
}

pub trait EffectProps {
    const NAME: &'static str;
    const PROPERTIES: &[PropertyBinding];
}

trait PropertyTypeName {
    const TYPE_NAME: &'static str;
}

impl PropertyTypeName for f32 {
    const TYPE_NAME: &'static str = "float";
}

impl PropertyTypeName for Vec2f {
    const TYPE_NAME: &'static str = "vector2";
}

impl PropertyTypeName for Vec3f {
    const TYPE_NAME: &'static str = "vector3";
}

impl PropertyTypeName for Vec4f {
    const TYPE_NAME: &'static str = "vector4";
}

macro_rules! custom_effect {
    (
	$sv:vis struct $sname:ident { $($fv:vis $fname:ident : $ftype:ty),* $(,)? }
	) => {
        #[repr(C)]
        #[derive(Clone, Copy)]
        $sv struct $sname {
            $(
				$fv $fname: $ftype
			),*
        }

		impl $crate::platform::win::filters::EffectProps for $sname {
            const NAME: &'static str = stringify!($sname);
            const PROPERTIES: &[PropertyBinding] = &[
                $(
                    property_binding!($sname, stringify!($fname), $fname, <$ftype as PropertyTypeName>::TYPE_NAME)
                ),*
            ];
        }

        unsafe impl NoUninit for $sname {}
	}
}

pub trait CustomEffect: EffectProps + NoUninit + 'static + Default {
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
pub struct EffectWrapper<T>
where
    T: CustomEffect,
{
    draw_info: RefCell<Option<Direct2D::ID2D1DrawInfo>>,
    consts: RefCell<T>,
}

impl<T: CustomEffect> EffectWrapper<T> {
    unsafe extern "system" fn create(
        effect_impl: windows_core::OutRef<'_, windows_core::IUnknown>,
    ) -> windows_core::HRESULT {
        let this = Self {
            draw_info: RefCell::new(None),
            consts: RefCell::new(T::default()),
        };
        let com_object: IUnknown = this.into();

        match effect_impl.write(Some(com_object)) {
            Ok(_) => windows_core::HRESULT::default(),
            Err(e) => e.into(),
        }
    }

    pub fn register(factory: &Direct2D::ID2D1Factory1) -> windows_core::Result<()> {
        // This is really unsound. We need wide string pointers in order to construct D2D1_PROPERTY_BINDINGs
        // But we have rust strings. So allocate a vector of HSTRINGS, and assign pointers into this vec
        // while creating the bindings.
        let prop_names: Vec<HSTRING> = T::PROPERTIES.iter().map(|prop| prop.name.into()).collect();
        let bindings: Vec<_> = T::PROPERTIES
            .iter()
            .zip(prop_names.iter())
            .map(|(prop, name)| Direct2D::D2D1_PROPERTY_BINDING {
                propertyName: PCWSTR(name.as_ptr()),
                setFunction: prop.setter,
                getFunction: prop.getter,
            })
            .collect();

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
                <!-- Custom Properties go here -->",
            T::NAME
        )
        .expect("Unable to format CustomEffect xml");

        for prop in T::PROPERTIES.iter() {
            xml.push('\n');
            xml.push_str(&prop.xml_str());
        }
        xml.push_str("\n</Effect>");

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

    fn PrepareForRender(
        &self,
        _changetype: Direct2D::D2D1_CHANGE_TYPE,
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

impl<T: CustomEffect> Direct2D::ID2D1TransformNode_Impl for EffectWrapper_Impl<T> {
    fn GetInputCount(&self) -> u32 {
        0
    }
}

const DEFAULT_SHADOW_COLOR: Vec4f = Vec4f {
    x: 0.0,
    y: 0.0,
    z: 0.0,
    w: 0.3,
};

custom_effect!(
    pub struct RectShadowEffectShader {
        size: Vec2f,
        shadow_radius: f32,
        shadow_offset: Vec2f,
        shadow_color: Vec4f,
    }
);

impl CustomEffect for RectShadowEffectShader {
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
        todo!()
    }
}

impl Default for RectShadowEffectShader {
    fn default() -> Self {
        Self {
            size: Default::default(),
            shadow_radius: Default::default(),
            shadow_offset: Default::default(),
            shadow_color: DEFAULT_SHADOW_COLOR,
        }
    }
}

custom_effect!(
    pub struct RoundedRectShadowEffectShader {
        shadow_color: Vec4f,
        size: Vec2f,
        offset: Vec2f,
        corner_radius: f32,
        shadow_radius: f32,
    }
);

impl CustomEffect for RoundedRectShadowEffectShader {
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
        let top_left = Vec2f::ZERO.min(self.offset).floor();
        let bottom_right = self.size + Vec2f::ZERO.max(self.offset).ceil();
        RECT {
            left: top_left.x as i32,
            top: top_left.y as i32,
            right: bottom_right.x as i32,
            bottom: bottom_right.y as i32,
        }
    }
}

impl Default for RoundedRectShadowEffectShader {
    fn default() -> Self {
        Self {
            shadow_color: DEFAULT_SHADOW_COLOR,
            size: Vec2f { x: 1.0, y: 1.0 },
            offset: Vec2f { x: 0.0, y: 0.0 },
            corner_radius: 0.0,
            shadow_radius: 0.0,
        }
    }
}

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
