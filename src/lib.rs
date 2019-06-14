extern crate core_foundation;
extern crate core_foundation_sys;
#[macro_use]
extern crate bitflags;
extern crate libc;

use core_foundation::array::{CFArray, CFArrayRef};
use core_foundation::base::TCFType;
use core_foundation::error::{CFError, CFErrorRef};
use core_foundation::string::{CFString, CFStringRef};
use core_foundation::url::{CFURLRef, CFURL};

use core_foundation_sys::base::OSStatus;

use libc::c_void;

pub type OptionBits = u32;

#[repr(C)]
struct PrimitiveLSLaunchURLSpec {
    app_url: CFURLRef,
    item_urls: CFArrayRef,
    pass_thru_params: *const c_void,
    launch_flags: OptionBits,
    async_ref_con: *const c_void,
}

#[link(name = "CoreServices", kind = "framework")]
extern "C" {
    fn LSCopyDefaultApplicationURLForURL(
        inURL: CFURLRef,
        inRoleMask: OptionBits,
        outError: *mut CFErrorRef,
    ) -> CFURLRef;
    fn LSCopyDefaultApplicationURLForContentType(
        inURL: CFStringRef,
        inRoleMask: OptionBits,
        outError: *mut CFErrorRef,
    ) -> CFURLRef;
    fn LSCopyApplicationURLsForURL(inURL: CFStringRef, inRoleMask: OptionBits) -> CFArrayRef;
    fn LSCanURLAcceptURL(
        inItemURL: CFURLRef,
        inTargetURL: CFURLRef,
        inRoleMask: OptionBits,
        inFlags: OptionBits,
        outAcceptsItem: &mut bool,
    ) -> OSStatus;
    fn LSCopyApplicationURLsForBundleIdentifier(
        inBundleIdentifier: CFStringRef,
        outError: *mut CFErrorRef,
    ) -> CFArrayRef;
    fn LSOpenCFURLRef(inURL: CFURLRef, outLaunchedURL: *mut CFURLRef) -> OSStatus;
    fn LSOpenFromURLSpec(
        inLaunchSpec: *const PrimitiveLSLaunchURLSpec,
        outLaunchedURL: *mut CFURLRef,
    ) -> OSStatus;
    fn LSRegisterURL(inURL: CFURLRef, inUpdate: bool) -> OSStatus;
    fn LSCopyAllRoleHandlersForContentType(
        inContentType: CFStringRef,
        inRole: OptionBits,
    ) -> CFArrayRef;
    fn LSCopyDefaultRoleHandlerForContentType(
        inContentType: CFStringRef,
        inRole: OptionBits,
    ) -> CFStringRef;
    fn LSSetDefaultRoleHandlerForContentType(
        inContentType: CFStringRef,
        inRole: OptionBits,
        inHandlerBundleID: CFStringRef,
    ) -> OSStatus;
    fn LSSetDefaultHandlerForURLScheme(
        inURLScheme: CFStringRef,
        inHandlerBundleID: CFStringRef,
    ) -> OSStatus;
}

bitflags! {
    pub struct LSRolesMask: OptionBits {
        const NONE      = 0x00000001;
        const VIEWER    = 0x00000002;
        const EDITOR    = 0x00000004;
        const SHELL     = 0x00000008;
        const ALL       = ::std::u32::MAX;
    }
}

bitflags! {
    pub struct LSAcceptanceFlags: OptionBits {
        const DEFAULT           = 0x00000001;
        const ALLOW_LOGIN_UI    = 0x00000002;
    }
}

bitflags! {
    pub struct LSLaunchFlags: OptionBits {
        const DEFAULTS              = 0x00000001;
        const PRINT                 = 0x00000002;
        const DISPLAY_ERRORS        = 0x00000040;
        const DONT_ADD_TO_RECENTS   = 0x00000100;
        const DONT_SWITCH           = 0x00000200;
        const ASYNC                 = 0x00010000;
        const NEW_INSTANCE          = 0x00080000;
        const HIDE                  = 0x00100000;
        const HIDE_OTHERS           = 0x00200000;
    }
}

pub struct LSLaunchURLSpec {
    pub app: Option<CFURL>,
    pub urls: Option<CFArray<CFURL>>,
    pub pass_thru_params: *const c_void,
    pub flags: LSLaunchFlags,
    pub async_ref_con: *const c_void,
}

impl LSLaunchURLSpec {
    pub(crate) fn to_primitive(&self) -> PrimitiveLSLaunchURLSpec {
        PrimitiveLSLaunchURLSpec {
            app_url: self.app.iter().next()
                .map(|v| v.as_concrete_TypeRef())
                .unwrap_or_else(std::ptr::null),
            item_urls: self.urls.iter().next()
                .map(|v| v.as_concrete_TypeRef())
                .unwrap_or_else(std::ptr::null),
            pass_thru_params: self.pass_thru_params,
            launch_flags: self.flags.bits(),
            async_ref_con: self.async_ref_con,
        }
    }
}

impl Default for LSLaunchURLSpec {
    fn default() -> Self {
        LSLaunchURLSpec {
            app: None,
            urls: None,
            pass_thru_params: std::ptr::null(),
            flags: LSLaunchFlags::DEFAULTS,
            async_ref_con: std::ptr::null(),
        }
    }
}

pub fn default_application_url_for_url(
    url: &CFURL,
    role_mask: LSRolesMask,
) -> Result<CFURL, CFError> {
    let mut err: CFErrorRef = std::ptr::null_mut();
    let res = unsafe {
        LSCopyDefaultApplicationURLForURL(url.as_concrete_TypeRef(), role_mask.bits(), &mut err)
    };

    if res.is_null() {
        Err(unsafe { TCFType::wrap_under_create_rule(err) })
    } else {
        Ok(unsafe { CFURL::wrap_under_create_rule(res) })
    }
}

pub fn default_application_url_content_type(
    url: &CFString,
    role_mask: LSRolesMask,
) -> Result<CFURL, CFError> {
    let mut err: CFErrorRef = std::ptr::null_mut();
    let res = unsafe {
        LSCopyDefaultApplicationURLForContentType(
            url.as_concrete_TypeRef(),
            role_mask.bits(),
            &mut err,
        )
    };

    if res.is_null() {
        Err(unsafe { TCFType::wrap_under_create_rule(err) })
    } else {
        Ok(unsafe { TCFType::wrap_under_create_rule(res) })
    }
}

pub fn application_urls_for_url(url: &CFString, role_mask: LSRolesMask) -> Option<CFArray<CFURL>> {
    let res = unsafe { LSCopyApplicationURLsForURL(url.as_concrete_TypeRef(), role_mask.bits()) };

    if res.is_null() {
        None
    } else {
        Some(unsafe { TCFType::wrap_under_create_rule(res) })
    }
}

pub fn can_url_accept_url(
    item_url: &CFURL,
    target_url: &CFURL,
    role_mask: LSRolesMask,
    flags: LSAcceptanceFlags,
) -> Result<bool, OSStatus> {
    let mut res: bool = false;
    let err = unsafe {
        LSCanURLAcceptURL(
            item_url.as_concrete_TypeRef(),
            target_url.as_concrete_TypeRef(),
            role_mask.bits(),
            flags.bits(),
            &mut res,
        )
    };

    if err == 0 {
        Ok(res)
    } else {
        Err(err)
    }
}

pub fn application_urls_for_bundle_identifier(
    bundle_identifier: &CFString,
) -> Result<CFArray<CFURL>, CFError> {
    let mut err: CFErrorRef = std::ptr::null_mut();
    let res = unsafe {
        LSCopyApplicationURLsForBundleIdentifier(bundle_identifier.as_concrete_TypeRef(), &mut err)
    };

    if res.is_null() {
        Err(unsafe { TCFType::wrap_under_create_rule(err) })
    } else {
        Ok(unsafe { TCFType::wrap_under_create_rule(res) })
    }
}

pub fn open_url(url: &CFURL) -> Result<CFURL, OSStatus> {
    let mut res: CFURLRef = std::ptr::null_mut();
    let err = unsafe { LSOpenCFURLRef(url.as_concrete_TypeRef(), &mut res) };

    if err == 0 {
        Ok(unsafe { TCFType::wrap_under_create_rule(res) })
    } else {
        Err(err)
    }
}

pub fn open_from_url_spec(launch_spec: LSLaunchURLSpec) -> Result<CFURL, OSStatus> {
    let mut res: CFURLRef = std::ptr::null_mut();
    let err = unsafe { LSOpenFromURLSpec(&launch_spec.to_primitive(), &mut res) };

    if err == 0 {
        Ok(unsafe { TCFType::wrap_under_create_rule(res) })
    } else {
        Err(err)
    }
}

pub fn register_url(url: &CFURL, update: bool) -> Result<(), OSStatus> {
    let err = unsafe { LSRegisterURL(url.as_concrete_TypeRef(), update) };

    if err == 0 {
        Ok(())
    } else {
        Err(err)
    }
}

pub fn role_handlers_for_content_type(
    content_type: &CFString,
    role: LSRolesMask,
) -> Option<CFArray<CFString>> {
    let res = unsafe {
        LSCopyAllRoleHandlersForContentType(content_type.as_concrete_TypeRef(), role.bits())
    };

    if res.is_null() {
        None
    } else {
        Some(unsafe { TCFType::wrap_under_create_rule(res) })
    }
}

pub fn default_role_handler_for_content_type(
    content_type: &CFString,
    role: LSRolesMask,
) -> Option<CFString> {
    let res = unsafe {
        LSCopyDefaultRoleHandlerForContentType(content_type.as_concrete_TypeRef(), role.bits())
    };

    if res.is_null() {
        None
    } else {
        Some(unsafe { TCFType::wrap_under_create_rule(res) })
    }
}

pub fn set_default_role_handler_for_content_type(
    content_type: &CFString,
    role: LSRolesMask,
    bundle_id: &CFString,
) -> Result<(), OSStatus> {
    let err = unsafe {
        LSSetDefaultRoleHandlerForContentType(
            content_type.as_concrete_TypeRef(),
            role.bits(),
            bundle_id.as_concrete_TypeRef(),
        )
    };

    if err == 0 {
        Ok(())
    } else {
        Err(err)
    }
}

pub fn set_default_handle_for_url_scheme(
    url_scheme: &CFString,
    bundle_id: &CFString,
) -> Result<(), OSStatus> {
    let err = unsafe {
        LSSetDefaultHandlerForURLScheme(
            url_scheme.as_concrete_TypeRef(),
            bundle_id.as_concrete_TypeRef(),
        )
    };

    if err == 0 {
        Ok(())
    } else {
        Err(err)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        default_application_url_for_url, open_from_url_spec, LSLaunchFlags, LSLaunchURLSpec,
        LSRolesMask,
    };
    use core_foundation::array::CFArray;
    use core_foundation::base::TCFType;
    use core_foundation::string::{CFString, CFStringRef};
    use core_foundation::url::{CFURLRef, CFURL};
    use core_foundation_sys::base::{kCFAllocatorDefault, CFAllocatorRef};

    #[link(name = "CoreServices", kind = "framework")]
    extern "C" {
        fn CFURLCreateWithString(
            allocator: CFAllocatorRef,
            urlString: CFStringRef,
            baseURL: CFURLRef,
        ) -> CFURLRef;
    }

    fn url(url: &str) -> Option<CFURL> {
        let url = CFString::new(url);
        let ptr = unsafe {
            CFURLCreateWithString(
                kCFAllocatorDefault,
                url.as_concrete_TypeRef(),
                ::std::ptr::null(),
            )
        };

        if ptr.is_null() {
            None
        } else {
            Some(unsafe { CFURL::wrap_under_create_rule(ptr) })
        }
    }

    // #[test]
    fn it_works() {
        println!(
            "{:#?}",
            default_application_url_for_url(
                &url("http://www.google.com/").unwrap(),
                LSRolesMask::VIEWER
            )
        );
        println!(
            "{:#?}",
            default_application_url_for_url(
                &url("fail://a.big.fail").unwrap(),
                LSRolesMask::VIEWER
            )
        );
    }

    #[test]
    fn open_with_spec() {
        let scheme = url("https://").unwrap();
        let app = default_application_url_for_url(&scheme, LSRolesMask::VIEWER).unwrap();
        let urls = vec![
            url("https://news.ycombinator.com/").unwrap(),
            url("https://www.google.com/").unwrap(),
        ];
        let urls = CFArray::<CFURL>::from_CFTypes(&urls[..]);
        println!("{:#?}", app);
        let spec = LSLaunchURLSpec {
            app: Some(app),
            urls: Some(urls),
            flags: LSLaunchFlags::DEFAULTS | LSLaunchFlags::ASYNC,
            ..Default::default()
        };
        open_from_url_spec(spec).unwrap();
    }
}
