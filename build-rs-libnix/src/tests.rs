#[cfg(test)]
mod tests {
    use crate::{handle_content, link_search_directory};
    use std::env;
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_rustc_cfg_output() {
        let mut temp = NamedTempFile::new().unwrap();
        writeln!(
            temp,
            "cargo:rustc-cfg=freebsd11\ncargo:rustc-cfg=libc_const_extern_fn"
        )
        .unwrap();

        let content = fs::read_to_string(temp.path()).unwrap();
        let output = handle_content(content).unwrap();
        assert_eq!(
            output.rustc_arguments.join(" ").trim(),
            "--cfg 'freebsd11' --cfg 'libc_const_extern_fn'"
        );
    }

    #[test]
    fn test_rustc_link_search_kinds() {
        // no kind, as cortex-m-rt prints it: `all`, like cargo; each kind; a `=` in the path of a value without a kind
        let content = "cargo:rustc-link-search=/nix/store/x-cortex-m-rt\n\
            cargo:rustc-link-search=native=/nix/store/y\n\
            cargo:rustc-link-search=dependency=/nix/store/y\n\
            cargo:rustc-link-search=crate=/nix/store/y\n\
            cargo:rustc-link-search=framework=/nix/store/y\n\
            cargo:rustc-link-search=all=/nix/store/y\n\
            cargo:rustc-link-search=/nix/store/a=b\n"
            .to_string();
        let output = handle_content(content).unwrap();
        assert_eq!(
            output.rustc_arguments,
            [
                "-L \"all=/nix/store/x-cortex-m-rt\"",
                "-L \"native=/nix/store/y\"",
                "-L \"dependency=/nix/store/y\"",
                "-L \"crate=/nix/store/y\"",
                "-L \"framework=/nix/store/y\"",
                "-L \"all=/nix/store/y\"",
                "-L \"all=/nix/store/a=b\"",
            ]
        );
        assert_eq!(
            output.rustc_propagated_arguments[0..2],
            ["-L '/nix/store/x-cortex-m-rt'", "-L 'native=/nix/store/y'"]
        );
    }

    #[test]
    fn test_link_search_directory() {
        let out = Some("/nix/store/h-run");
        assert_eq!(link_search_directory("/nix/store/h-run", out), "$out");
        assert_eq!(link_search_directory("/nix/store/h-run/", out), "$out");
        assert_eq!(link_search_directory("/nix/store/h-run/install/lib", out), "$out/install/lib");
        // only whole path components: another store path
        assert_eq!(link_search_directory("/nix/store/h-run-2/lib", out), "/nix/store/h-run-2/lib");
        assert_eq!(link_search_directory("/nix/store/h-sqlite/lib", out), "/nix/store/h-sqlite/lib");
        assert_eq!(link_search_directory("/build/tmp.X3Lovygu3U", out), "$out");
        assert_eq!(link_search_directory("relative/dir", out), "$out");
        assert_eq!(link_search_directory("/build/tmp.X3Lovygu3U", None), "$out");
    }

    #[test]
    fn test_rustc_link_arg_bins() {
        // only for the bin targets: neither in the rustc arguments of the package nor propagated
        let content = "cargo:rustc-link-arg-bins=--nmagic\ncargo:rustc-link-arg-bins=-Tlink.x\n".to_string();
        let output = handle_content(content).unwrap();
        assert_eq!(output.rustc_link_arg_bins.join(" "), "-C link-arg='--nmagic' -C link-arg='-Tlink.x'");
        assert!(output.rustc_arguments.is_empty());
        assert!(output.rustc_propagated_arguments.is_empty());
    }

    #[test]
    fn test_rustc_link_arg() {
        // tikv-jemalloc-sys
        let content = "cargo:rustc-link-lib=static=jemalloc_pic\ncargo:rustc-link-arg=-pthread\n".to_string();
        let output = handle_content(content).unwrap();
        assert_eq!(
            output.rustc_arguments.join(" ").trim(),
            "-l 'static=jemalloc_pic' -C link-arg='-pthread'"
        );
        assert!(output.rustc_propagated_arguments.is_empty());
    }

    #[test]
    fn test_output1() {
        let content = fs::read_to_string("test/output1").unwrap();
        let output = handle_content(content).unwrap();
        assert_eq!(
            output.rustc_arguments.join(" ").trim(),
            "--cfg 'libc_const_extern_fn' --cfg 'freebsd11' --check-cfg 'cfg(espidf_time32)' --check-cfg 'cfg(target_arch,values(\"mips64r6\"))'"
        );
    }

    #[test]
    fn test_output2() {
        let content = fs::read_to_string("test/output2").unwrap();
        let output = handle_content(content).unwrap();
        assert_eq!(
            output.rustc_arguments.join(" ").trim(),
            "--check-cfg 'cfg(fast_arithmetic, values(\"32\", \"64\"))' --cfg 'fast_arithmetic=\"64\"'"
        );
    }

    #[test]
    fn test_output3() {
        let content = fs::read_to_string("test/output3").unwrap();
        let output = handle_content(content).unwrap();
        assert_eq!(
            output.rustc_arguments.join(" ").trim(),
            "-l 'static=sqlite3' -L \"native=$out\""
        );
        assert_eq!(
            output.rustc_propagated_arguments.join(" ").trim(),
            "-L 'native=${rust-embed-8_6_0-50d2bdadc507cf36}'"
        );
    }

    #[test]
    fn test_output4() {
        let content = fs::read_to_string("test/output4").unwrap();
        let err = handle_content(content).unwrap_err();
        assert!(err.to_string().contains("cargo:fail here"));
    }

    #[test]
    fn test_output5() {
        unsafe {
            env::set_var("CARGO_MANIFEST_LINKS", "MYCRATE");
        }
        let content = fs::read_to_string("test/output5").unwrap();
        let output = handle_content(content).unwrap();
        assert_eq!(
            output.rustc_arguments.join(" ").trim(),
            "--check-cfg 'cfg(openssl)' --check-cfg 'cfg(libressl)' --check-cfg 'cfg(boringssl)' --check-cfg 'cfg(libressl250)' --check-cfg 'cfg(libressl251)' --check-cfg 'cfg(libressl252)' --check-cfg 'cfg(libressl261)' --check-cfg 'cfg(libressl270)' --check-cfg 'cfg(libressl271)' --check-cfg 'cfg(libressl273)' --check-cfg 'cfg(libressl280)' --check-cfg 'cfg(libressl281)' --check-cfg 'cfg(libressl291)' --check-cfg 'cfg(libressl310)' --check-cfg 'cfg(libressl321)' --check-cfg 'cfg(libressl332)' --check-cfg 'cfg(libressl340)' --check-cfg 'cfg(libressl350)' --check-cfg 'cfg(libressl360)' --check-cfg 'cfg(libressl361)' --check-cfg 'cfg(libressl370)' --check-cfg 'cfg(libressl380)' --check-cfg 'cfg(libressl381)' --check-cfg 'cfg(libressl382)' --check-cfg 'cfg(libressl390)' --check-cfg 'cfg(libressl400)' --check-cfg 'cfg(libressl410)' --check-cfg 'cfg(ossl101)' --check-cfg 'cfg(ossl102)' --check-cfg 'cfg(ossl102f)' --check-cfg 'cfg(ossl102h)' --check-cfg 'cfg(ossl110)' --check-cfg 'cfg(ossl110f)' --check-cfg 'cfg(ossl110g)' --check-cfg 'cfg(ossl110h)' --check-cfg 'cfg(ossl111)' --check-cfg 'cfg(ossl111b)' --check-cfg 'cfg(ossl111c)' --check-cfg 'cfg(ossl111d)' --check-cfg 'cfg(ossl300)' --check-cfg 'cfg(ossl310)' --check-cfg 'cfg(ossl320)' --check-cfg 'cfg(ossl330)' --check-cfg 'cfg(ossl340)' -L \"native=/nix/store/byx7ahs386pskh8d5sdkrkpscfz9yyjp-openssl-3.4.1/lib\" -l 'ssl' -l 'crypto' --cfg 'osslconf=\"OPENSSL_NO_SSL3_METHOD\"' --cfg 'openssl' --cfg 'ossl340' --cfg 'ossl330' --cfg 'ossl320' --cfg 'ossl300' --cfg 'ossl101' --cfg 'ossl102' --cfg 'ossl102f' --cfg 'ossl102h' --cfg 'ossl110' --cfg 'ossl110f' --cfg 'ossl110g' --cfg 'ossl110h' --cfg 'ossl111' --cfg 'ossl111b' --cfg 'ossl111c' --cfg 'ossl111d'"
        );
        assert_eq!(
            output.rustc_propagated_arguments.join(" ").trim(),
            "-L 'native=/nix/store/byx7ahs386pskh8d5sdkrkpscfz9yyjp-openssl-3.4.1/lib'"
        );
        assert_eq!(
            output.environment_variables.join("\n").trim(),
            "DEP_MYCRATE_CONF='OPENSSL_NO_SSL3_METHOD'\nDEP_MYCRATE_VERSION_NUMBER='30400010'\nDEP_MYCRATE_INCLUDE='/nix/store/k0699a27nkj4c2xn67bjcpfa08nqn9l4-openssl-3.4.1-dev/include'"
        );
    }

    #[test]
    fn test_output6() {
        let content = fs::read_to_string("test/output6").unwrap();
        let output = handle_content(content).unwrap();
    }

    #[test]
    fn test_output7() {
        unsafe {
            env::set_var("CARGO_MANIFEST_LINKS", "MYCRATE");
        }
        let content = fs::read_to_string("test/output7").unwrap();
        let output = handle_content(content).unwrap();
        assert_eq!(
            output.rustc_arguments.join(" ").trim(),
            "-L \"native=/nix/store/yfjzkkkyxcalyj7l1n4d4y6s81i65hmy-sqlite-3.48.0/lib\" -l 'sqlite3'"
        );
        assert_eq!(
            output.rustc_propagated_arguments.join(" ").trim(),
            "-L 'native=/nix/store/yfjzkkkyxcalyj7l1n4d4y6s81i65hmy-sqlite-3.48.0/lib'"
        );
        // `cargo:link-target=sqlite3` is metadata like any other key cargo doesn't know
        assert_eq!(output.environment_variables.join("\n").trim(), "DEP_MYCRATE_LINK_TARGET='sqlite3'");
        assert_eq!(
            output.rustc_link_arg_benches.join(" ").trim(),
            "-C link-arg='-rdynamic'"
        );
    }

    #[test]
    fn test_output8() {
        // blake3 1.8.7: `cargo::` instructions (Rust 1.77+) mixed with the `cargo:` ones of the cc crate
        let content = fs::read_to_string("test/output8").unwrap();
        let output = handle_content(content).unwrap();
        let check_cfgs: Vec<String> = [
            "sse2_ffi", "sse2_rust", "sse41_ffi", "sse41_rust", "avx2_ffi", "avx2_rust", "avx512_ffi", "neon",
            "wasm32_simd",
        ]
        .iter()
        .map(|c| format!("--check-cfg 'cfg(blake3_{c}, values(none()))'"))
        .collect();
        assert_eq!(
            output.rustc_arguments.join(" ").trim(),
            format!(
                "{} --cfg 'blake3_sse2_ffi' --cfg 'blake3_sse41_ffi' --cfg 'blake3_avx2_ffi' \
                 -l 'static=blake3_sse2_sse41_avx2_assembly' -L \"native=/nix/store/x3bz4fjz2xqkmq2k6z00pnzd2fci606c-blake3-1_8_7-script_build_run-b094bfd45a5d122f\" --cfg 'blake3_avx512_ffi' \
                 -l 'static=blake3_avx512_assembly' -L \"native=/nix/store/x3bz4fjz2xqkmq2k6z00pnzd2fci606c-blake3-1_8_7-script_build_run-b094bfd45a5d122f\"",
                check_cfgs.join(" ")
            )
        );
    }

    #[test]
    fn test_output9() {
        // aws-lc-sys 0.47.0: metadata with keys of its own and an empty value (`cargo:system_libs=`)
        unsafe {
            env::set_var("CARGO_MANIFEST_LINKS", "MYCRATE");
        }
        let content = fs::read_to_string("test/output9").unwrap();
        let output = handle_content(content).unwrap();
        let out = "/nix/store/mrkw2zlbxq94xssqb7l85lfm43jcd0kk-aws-lc-sys-0_47_0-script_build_run-6ff63a6c3919aaf8";
        assert_eq!(
            output.rustc_arguments.join(" ").trim(),
            format!("-l 'static=aws_lc_0_47_0_crypto' -L \"native={out}\"")
        );
        let envs: Vec<&str> = output.environment_variables.iter().map(|e| e.as_str()).collect();
        assert_eq!(envs[0], format!("DEP_MYCRATE_LIBDIR='{out}'"));
        assert_eq!(envs[1], "DEP_MYCRATE_LINK_KIND='static'");
        assert_eq!(envs[2], format!("DEP_MYCRATE_LIBCRYPTO_PATH='{out}/libaws_lc_0_47_0_crypto.a'"));
        assert_eq!(envs[3], "DEP_MYCRATE_SYSTEM_LIBS=''");
        assert_eq!(envs[4], format!("DEP_MYCRATE_INCLUDE='{out}/include'"));
        assert_eq!(envs[5], "DEP_MYCRATE_LIBCRYPTO='aws_lc_0_47_0_crypto'");
        assert!(envs[6].starts_with("DEP_MYCRATE_CONF='OPENSSL_NO_ASYNC,OPENSSL_NO_BF,"));
        assert_eq!(envs.len(), 7);
    }

    #[test]
    fn test_metadata_empty_value_and_unknown_keys() {
        unsafe {
            env::set_var("CARGO_MANIFEST_LINKS", "MYCRATE");
        }
        let content = "cargo::metadata=system_libs=\ncargo:some-key=x\ncargo::unknown-directive=y\n".to_string();
        let output = handle_content(content).unwrap();
        // keys are envified like cargo does: upper case, `-` -> `_`; unknown `cargo::` directives are no metadata
        assert_eq!(
            output.environment_variables.join("\n"),
            "DEP_MYCRATE_SYSTEM_LIBS=''\nDEP_MYCRATE_SOME_KEY='x'"
        );
        let err = handle_content("cargo::metadata==value\n".to_string()).unwrap_err();
        assert!(err.to_string().contains("Unable to parse metadata argument"));
    }

    #[test]
    fn test_new_syntax() {
        unsafe {
            env::set_var("CARGO_MANIFEST_LINKS", "MYCRATE");
        }
        let content = "cargo::rustc-cfg=foo\ncargo:rustc-cfg=bar\ncargo::rustc-env=A=b\n\
                       cargo::rustc-link-lib=z\ncargo::rerun-if-changed=build.rs\n\
                       cargo::metadata=include=/inc\ncargo:root=/root\n"
            .to_string();
        let output = handle_content(content).unwrap();
        assert_eq!(output.rustc_arguments.join(" ").trim(), "--cfg 'foo' --cfg 'bar' -l 'z'");
        assert_eq!(
            output.environment_variables.join("\n").trim(),
            "A='b'\nDEP_MYCRATE_INCLUDE='/inc'\nDEP_MYCRATE_ROOT='/root'"
        );
        let err = handle_content("cargo::metadata=novalue\n".to_string()).unwrap_err();
        assert!(err.to_string().contains("Unable to parse metadata argument"));
        let err = handle_content("cargo::error=new syntax\n".to_string()).unwrap_err();
        assert_eq!(err.to_string(), "build.rs reported 1 error(s) via 'cargo:error'");
    }

    #[test]
    fn test_error_fails() {
        // every error is reported, then the build fails, like cargo does
        let content = "cargo:error=first\ncargo:rustc-cfg=foo\ncargo:error=second\n".to_string();
        let err = handle_content(content).unwrap_err();
        assert_eq!(err.to_string(), "build.rs reported 2 error(s) via 'cargo:error'");
    }
}
