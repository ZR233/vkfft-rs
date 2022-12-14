use std::env;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;
use bindgen::Bindings;
extern crate bindgen;
extern crate cc;
extern crate core;

#[derive(Default)]
struct BuildArgs{
    out_dir: PathBuf,
    manifest_dir: PathBuf,
    toolchain_file: PathBuf,
    target_os: String,
    android_abi: String,
}


fn main() -> Result<(), Box<dyn Error>> {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let vkfft_root = PathBuf::from(&manifest_dir).join("VkFFT-1.2.31");
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let mut build_args = BuildArgs::default();

    println!("cargo:rerun-if-changed=wrapper/wrapper.cpp");
    println!("cargo:rerun-if-changed=build.rs");


    build_args.out_dir = out_dir;
    build_args.manifest_dir = manifest_dir;
    build_args.target_os = target_os;

    if build_args.target_os == "android" {
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
        build_args.android_abi = android_abi.to_string();

        let ndk_home = PathBuf::from(env::var("ANDROID_NDK_HOME").expect("ANDROID_NDK_HOME not set!"));
        build_args.toolchain_file = ndk_home.join("build").join("cmake").join("android.toolchain.cmake");
    }


    build_glslang(&build_args);


    // 生成wrapper
    let wrapper = std::fs::read_to_string(format!("{}/vkFFT/vkFFT.h", build_args.manifest_dir
        .join("VkFFT-1.2.31").display()))?
        .replace("static inline", "");
    let rw = build_args.out_dir.join("include").join("vkfft_rw.hpp");
    std::fs::write(&rw, wrapper.as_str())?;



    build_vkfft(&build_args);

    let mut libraries = vec![
        "glslang",
        "MachineIndependent",
        "OSDependent",
        "GenericCodeGen",
        "OGLCompiler",
        "SPIRV",
        "VkFFT"
    ];

    if build_args.target_os=="windows" {
        libraries.push("vulkan-1");
    }else {
        libraries.push("vulkan");
    }

    let lib_dir = build_args.out_dir.join("lib").display().to_string();

    println!("cargo:rustc-link-search={}", lib_dir.clone());
    for library in libraries.iter() {
        println!("cargo:rustc-link-lib={}", library);
    }

    let include_dirs = [
        format!("{}/vkFFT", vkfft_root.display()),
        format!("{}/include/glslang/Include", build_args.out_dir.display()),
        format!("{}", build_args.out_dir.join("include").display())
    ];
    let defines = [
        ("VKFFT_BACKEND", "0"),
        ("VK_API_VERSION", "11")
    ];

    // let wrapper_h = build_args.manifest_dir.join("wrapper").join("wrapper.h");

    // let bindings = gen_wrapper(&build_args, &wrapper_h, &defines, &include_dirs)?;
    // bindings.write_to_file(build_args.out_dir.join("bindings.rs"))?;

    Ok(())
}

fn build_glslang(build_args: &BuildArgs){
    let mut cmd = Command::new("cmake");
    let cmake_build_dir = build_args.out_dir.join("build").join("glslang_master");
    let src = build_args.manifest_dir.join("glslang-master");
    cmd.arg("-S").arg(src);
    cmd.arg("-B").arg(cmake_build_dir.clone());
    cmd.arg(format!("-DCMAKE_INSTALL_PREFIX={}", build_args.out_dir.display()));
    cmd.arg("-DCMAKE_BUILD_TYPE=Release");

    if build_args.target_os=="android" {
        cmd.arg(format!("-DANDROID_ABI={}", build_args.android_abi));
        cmd.arg("-DANDROID_STL=c++_static");
        cmd.arg("-DANDROID_PLATFORM=android-24");
        cmd.arg("-G").arg("Ninja");
        cmd.arg(format!("-DCMAKE_TOOLCHAIN_FILE={}", build_args.toolchain_file.display()));
    }

    run(cmd);
    let mut cmd= Command::new("cmake");

    cmd.arg("--build").arg(cmake_build_dir);
    cmd.args(["--config", "Release", "--target", "install"]);


    run(cmd)
}
fn build_vkfft(build_args: &BuildArgs){
    let mut cmd = Command::new("cmake");
    let cmake_build_dir = build_args.out_dir.join("build").join("vkfft");
    let src = build_args.manifest_dir.join("wrapper");
    cmd.arg("-S").arg(src);
    cmd.arg("-B").arg(cmake_build_dir.clone());
    cmd.arg(format!("-DCMAKE_INSTALL_PREFIX={}", build_args.out_dir.display()));
    cmd.arg("-DCMAKE_BUILD_TYPE=Release");

    if build_args.target_os=="android" {
        cmd.arg(format!("-DANDROID_ABI={}", build_args.android_abi));
        cmd.arg("-DANDROID_STL=c++_static");
        cmd.arg("-DANDROID_PLATFORM=android-26");
        cmd.arg("-G").arg("Ninja");
        cmd.arg(format!("-DCMAKE_TOOLCHAIN_FILE={}", build_args.toolchain_file.display()));
    }

    run(cmd);
    let mut cmd= Command::new("cmake");

    cmd.arg("--build").arg(cmake_build_dir);
    cmd.args(["--config", "Release", "--target", "install"]);


    run(cmd)
}

fn run(mut cmd: Command){
    println!("running: {:?}", cmd);
    let status = cmd.status().unwrap();
    if !status.success() {
        panic!("run cmd fail: {:?}", cmd)
    }
}

fn gen_wrapper<F, const N: usize, const M: usize>(build_args: &BuildArgs, file: F,  defines: &[(&str, &str); N], include_dirs: &[String; M]) -> Result<Bindings, Box<dyn Error>>
    where
        F: AsRef<Path>,
{
    let base_args = [
        // "-std=c++11".to_string(),
        // "-std=c11".to_string(),
    ];

    let defines: Vec<String> = defines.iter().map(|(k, v)| {
        format!("-D{}={}", k, v)
    }).collect();

    let include_dirs: Vec<String> = include_dirs.iter()
        .map(|s| format!("-I{}", s))
        .collect();

    let clang_args = base_args
        .iter()
        .chain(defines.iter())
        .chain(include_dirs.iter());


    let mut cfg = bindgen::Builder::default();

    if build_args.target_os == "android" {
        let mut sys_path = PathBuf::from(env::var("ANDROID_NDK_HOME").unwrap());
        sys_path = sys_path.join("toolchains/llvm/prebuilt");
        //TODO 只有win
        sys_path = sys_path.join("windows-x86_64");

        sys_path = sys_path.join("sysroot");

        cfg = cfg.clang_arg("--sysroot").clang_arg(sys_path.display().to_string());
    }



    let cfg = cfg
        .clang_args(clang_args)
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .header(file.as_ref().to_str().unwrap())
        .allowlist_recursively(true)
        .allowlist_type("VkFFTConfiguration")
        .allowlist_type("VkFFTLaunchParams")
        .allowlist_type("VkFFTResult")
        .allowlist_type("VkFFTSpecializationConstantsLayout")
        .allowlist_type("VkFFTPushConstantsLayout")
        .allowlist_type("VkFFTAxis")
        .allowlist_type("VkFFTPlan")
        .allowlist_type("VkFFTApplication")
        .allowlist_function("VkFFTSync")
        .allowlist_function("VkFFTAppend")
        .allowlist_function("VkFFTPlanAxis")
        .allowlist_function("initializeVkFFT")
        .allowlist_function("deleteVkFFT")
        .allowlist_function("VkFFTGetVersion");

    println!("{:?}", cfg);

    let res = cfg.generate();

    let bindings = match res {
        Ok(x) => x,
        Err(_) => {
            eprintln!("Failed to generate bindings.");
            std::process::exit(1);
        }
    };

    Ok(bindings)
}