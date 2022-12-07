use std::error::Error;
use std::env;
use std::path::PathBuf;
use cmake::Config;


fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let mut libraries = vec![
        "glslang",
        "MachineIndependent",
        "OSDependent",
        "GenericCodeGen",
        "OGLCompiler",
        "SPIRV"
    ];

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();

    let mut cfg = Config::new("VkFFT-1.2.31");
    cfg.profile("Release");
    if target_os == "android" {
        let ndk_home = env::var("ANDROID_NDK_HOME").expect("ANDROID_NDK_HOME not set!");
        cfg.generator("Ninja");

        let toolchain_file = PathBuf::from(ndk_home.as_str()).join("build").join("cmake").join("android.toolchain.cmake");

        cfg.define("CMAKE_TOOLCHAIN_FILE", toolchain_file);
        cfg.define("ANDROID_PLATFORM", "26");
        let arch = env::var("CARGO_CFG_TARGET_ARCH").unwrap();
        let android_abi;
        match  arch.as_str(){
            "x86" => {
                android_abi="x86"
            }
            "x86_64" => {
                android_abi="x86_64"
            }
            "arm" => {
                android_abi="armeabi-v7a"
            }
            "aarch64" => {
                android_abi="arm64-v8a"
            }
            _ =>{
                panic!("arch not support: {}", arch);
            }
        }

        cfg.define("ANDROID_ABI", android_abi);

        libraries.push("vulkan");
    }


    cfg.define("build_VkFFT_FFTW_precision", "off");

    let dst = cfg
        .build();

    println!("cargo:rustc-link-search=native={}", dst.join("lib").display());

    if target_os=="windows" {
        libraries.push("vulkan-1");
        let vulkan_home = env::var("VK_SDK_PATH").expect("VK_SDK_PATH not set!");
        let vulkan_lib_dir = PathBuf::from(vulkan_home).join("Lib");

        println!("cargo:rustc-link-search=native={}",  vulkan_lib_dir.display());
    }



    for library in libraries.iter() {
        println!("cargo:rustc-link-lib=static={}", library);
    }




    Ok(())
}