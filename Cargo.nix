# This file was @generated by cargo2nix 0.11.0.
# It is not intended to be manually edited.

args@{
  release ? true,
  rootFeatures ? [
    "clap4shell/default"
  ],
  rustPackages,
  buildRustPackages,
  hostPlatform,
  hostPlatformCpu ? null,
  hostPlatformFeatures ? [],
  target ? null,
  codegenOpts ? null,
  profileOpts ? null,
  rustcLinkFlags ? null,
  rustcBuildFlags ? null,
  mkRustCrate,
  rustLib,
  lib,
  workspaceSrc,
}:
let
  workspaceSrc = if args.workspaceSrc == null then ./. else args.workspaceSrc;
in let
  inherit (rustLib) fetchCratesIo fetchCrateLocal fetchCrateGit fetchCrateAlternativeRegistry expandFeatures decideProfile genDrvsByProfile;
  profilesByName = {
  };
  rootFeatures' = expandFeatures rootFeatures;
  overridableMkRustCrate = f:
    let
      drvs = genDrvsByProfile profilesByName ({ profile, profileName }: mkRustCrate ({ inherit release profile hostPlatformCpu hostPlatformFeatures target profileOpts codegenOpts rustcLinkFlags rustcBuildFlags; } // (f profileName)));
    in { compileMode ? null, profileName ? decideProfile compileMode release }:
      let drv = drvs.${profileName}; in if compileMode == null then drv else drv.override { inherit compileMode; };
in
{
  cargo2nixVersion = "0.11.0";
  workspace = {
    clap4shell = rustPackages.unknown.clap4shell."0.3.2";
  };
  "registry+https://github.com/rust-lang/crates.io-index".atty."0.2.14" = overridableMkRustCrate (profileName: rec {
    name = "atty";
    version = "0.2.14";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "d9b39be18770d11421cdb1b9947a45dd3f37e93092cbf377614828a319d5fee8"; };
    dependencies = {
      ${ if hostPlatform.parsed.kernel.name == "hermit" then "hermit_abi" else null } = rustPackages."registry+https://github.com/rust-lang/crates.io-index".hermit-abi."0.1.19" { inherit profileName; };
      ${ if hostPlatform.isUnix then "libc" else null } = rustPackages."registry+https://github.com/rust-lang/crates.io-index".libc."0.2.132" { inherit profileName; };
      ${ if hostPlatform.isWindows then "winapi" else null } = rustPackages."registry+https://github.com/rust-lang/crates.io-index".winapi."0.3.9" { inherit profileName; };
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".autocfg."1.1.0" = overridableMkRustCrate (profileName: rec {
    name = "autocfg";
    version = "1.1.0";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "d468802bab17cbc0cc575e9b053f41e72aa36bfa6b7f55e3529ffa43161b97fa"; };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".bitflags."1.3.2" = overridableMkRustCrate (profileName: rec {
    name = "bitflags";
    version = "1.3.2";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "bef38d45163c2f1dde094a7dfd33ccf595c92905c8f8f4fdc18d06fb1037718a"; };
    features = builtins.concatLists [
      [ "default" ]
    ];
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".clap."3.2.17" = overridableMkRustCrate (profileName: rec {
    name = "clap";
    version = "3.2.17";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "29e724a68d9319343bb3328c9cc2dfde263f4b3142ee1059a9980580171c954b"; };
    features = builtins.concatLists [
      [ "atty" ]
      [ "color" ]
      [ "default" ]
      [ "env" ]
      [ "std" ]
      [ "strsim" ]
      [ "suggestions" ]
      [ "termcolor" ]
    ];
    dependencies = {
      atty = rustPackages."registry+https://github.com/rust-lang/crates.io-index".atty."0.2.14" { inherit profileName; };
      bitflags = rustPackages."registry+https://github.com/rust-lang/crates.io-index".bitflags."1.3.2" { inherit profileName; };
      clap_lex = rustPackages."registry+https://github.com/rust-lang/crates.io-index".clap_lex."0.2.4" { inherit profileName; };
      indexmap = rustPackages."registry+https://github.com/rust-lang/crates.io-index".indexmap."1.9.1" { inherit profileName; };
      strsim = rustPackages."registry+https://github.com/rust-lang/crates.io-index".strsim."0.10.0" { inherit profileName; };
      termcolor = rustPackages."registry+https://github.com/rust-lang/crates.io-index".termcolor."1.1.3" { inherit profileName; };
      textwrap = rustPackages."registry+https://github.com/rust-lang/crates.io-index".textwrap."0.15.0" { inherit profileName; };
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".clap-serde."0.5.1" = overridableMkRustCrate (profileName: rec {
    name = "clap-serde";
    version = "0.5.1";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "f1d9609b451fd69eec0ecd379932d605a64bba1746397b31455962c8ba2fdfb7"; };
    features = builtins.concatLists [
      [ "allow-deprecated" ]
      [ "default" ]
      [ "env" ]
      [ "snake-case-key" ]
    ];
    dependencies = {
      clap = rustPackages."registry+https://github.com/rust-lang/crates.io-index".clap."3.2.17" { inherit profileName; };
      serde = rustPackages."registry+https://github.com/rust-lang/crates.io-index".serde."1.0.144" { inherit profileName; };
    };
  });
  
  "unknown".clap4shell."0.3.2" = overridableMkRustCrate (profileName: rec {
    name = "clap4shell";
    version = "0.3.2";
    registry = "unknown";
    src = fetchCrateLocal workspaceSrc;
    dependencies = {
      clap = rustPackages."registry+https://github.com/rust-lang/crates.io-index".clap."3.2.17" { inherit profileName; };
      clap_serde = rustPackages."registry+https://github.com/rust-lang/crates.io-index".clap-serde."0.5.1" { inherit profileName; };
      clap_complete = rustPackages."registry+https://github.com/rust-lang/crates.io-index".clap_complete."3.2.4" { inherit profileName; };
      serde_yaml = rustPackages."registry+https://github.com/rust-lang/crates.io-index".serde_yaml."0.9.10" { inherit profileName; };
      yaml_rust = rustPackages."registry+https://github.com/rust-lang/crates.io-index".yaml-rust."0.4.5" { inherit profileName; };
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".clap_complete."3.2.4" = overridableMkRustCrate (profileName: rec {
    name = "clap_complete";
    version = "3.2.4";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "e4179da71abd56c26b54dd0c248cc081c1f43b0a1a7e8448e28e57a29baa993d"; };
    features = builtins.concatLists [
      [ "default" ]
    ];
    dependencies = {
      clap = rustPackages."registry+https://github.com/rust-lang/crates.io-index".clap."3.2.17" { inherit profileName; };
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".clap_lex."0.2.4" = overridableMkRustCrate (profileName: rec {
    name = "clap_lex";
    version = "0.2.4";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "2850f2f5a82cbf437dd5af4d49848fbdfc27c157c3d010345776f952765261c5"; };
    dependencies = {
      os_str_bytes = rustPackages."registry+https://github.com/rust-lang/crates.io-index".os_str_bytes."6.3.0" { inherit profileName; };
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".hashbrown."0.12.3" = overridableMkRustCrate (profileName: rec {
    name = "hashbrown";
    version = "0.12.3";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "8a9ee70c43aaf417c914396645a0fa852624801b24ebb7ae78fe8272889ac888"; };
    features = builtins.concatLists [
      [ "raw" ]
    ];
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".hermit-abi."0.1.19" = overridableMkRustCrate (profileName: rec {
    name = "hermit-abi";
    version = "0.1.19";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "62b467343b94ba476dcb2500d242dadbb39557df889310ac77c5d99100aaac33"; };
    features = builtins.concatLists [
      [ "default" ]
    ];
    dependencies = {
      libc = rustPackages."registry+https://github.com/rust-lang/crates.io-index".libc."0.2.132" { inherit profileName; };
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".indexmap."1.9.1" = overridableMkRustCrate (profileName: rec {
    name = "indexmap";
    version = "1.9.1";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "10a35a97730320ffe8e2d410b5d3b69279b98d2c14bdb8b70ea89ecf7888d41e"; };
    features = builtins.concatLists [
      [ "std" ]
    ];
    dependencies = {
      hashbrown = rustPackages."registry+https://github.com/rust-lang/crates.io-index".hashbrown."0.12.3" { inherit profileName; };
    };
    buildDependencies = {
      autocfg = buildRustPackages."registry+https://github.com/rust-lang/crates.io-index".autocfg."1.1.0" { profileName = "__noProfile"; };
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".itoa."1.0.3" = overridableMkRustCrate (profileName: rec {
    name = "itoa";
    version = "1.0.3";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "6c8af84674fe1f223a982c933a0ee1086ac4d4052aa0fb8060c12c6ad838e754"; };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".libc."0.2.132" = overridableMkRustCrate (profileName: rec {
    name = "libc";
    version = "0.2.132";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "8371e4e5341c3a96db127eb2465ac681ced4c433e01dd0e938adbef26ba93ba5"; };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".linked-hash-map."0.5.6" = overridableMkRustCrate (profileName: rec {
    name = "linked-hash-map";
    version = "0.5.6";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "0717cef1bc8b636c6e1c1bbdefc09e6322da8a9321966e8928ef80d20f7f770f"; };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".os_str_bytes."6.3.0" = overridableMkRustCrate (profileName: rec {
    name = "os_str_bytes";
    version = "6.3.0";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "9ff7415e9ae3fff1225851df9e0d9e4e5479f947619774677a63572e55e80eff"; };
    features = builtins.concatLists [
      [ "raw_os_str" ]
    ];
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".proc-macro2."1.0.43" = overridableMkRustCrate (profileName: rec {
    name = "proc-macro2";
    version = "1.0.43";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "0a2ca2c61bc9f3d74d2886294ab7b9853abd9c1ad903a3ac7815c58989bb7bab"; };
    features = builtins.concatLists [
      [ "default" ]
      [ "proc-macro" ]
    ];
    dependencies = {
      unicode_ident = rustPackages."registry+https://github.com/rust-lang/crates.io-index".unicode-ident."1.0.3" { inherit profileName; };
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".quote."1.0.21" = overridableMkRustCrate (profileName: rec {
    name = "quote";
    version = "1.0.21";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "bbe448f377a7d6961e30f5955f9b8d106c3f5e449d493ee1b125c1d43c2b5179"; };
    features = builtins.concatLists [
      [ "default" ]
      [ "proc-macro" ]
    ];
    dependencies = {
      proc_macro2 = rustPackages."registry+https://github.com/rust-lang/crates.io-index".proc-macro2."1.0.43" { inherit profileName; };
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".ryu."1.0.11" = overridableMkRustCrate (profileName: rec {
    name = "ryu";
    version = "1.0.11";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "4501abdff3ae82a1c1b477a17252eb69cee9e66eb915c1abaa4f44d873df9f09"; };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".serde."1.0.144" = overridableMkRustCrate (profileName: rec {
    name = "serde";
    version = "1.0.144";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "0f747710de3dcd43b88c9168773254e809d8ddbdf9653b84e2554ab219f17860"; };
    features = builtins.concatLists [
      [ "default" ]
      [ "derive" ]
      [ "serde_derive" ]
      [ "std" ]
    ];
    dependencies = {
      serde_derive = buildRustPackages."registry+https://github.com/rust-lang/crates.io-index".serde_derive."1.0.144" { profileName = "__noProfile"; };
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".serde_derive."1.0.144" = overridableMkRustCrate (profileName: rec {
    name = "serde_derive";
    version = "1.0.144";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "94ed3a816fb1d101812f83e789f888322c34e291f894f19590dc310963e87a00"; };
    features = builtins.concatLists [
      [ "default" ]
    ];
    dependencies = {
      proc_macro2 = rustPackages."registry+https://github.com/rust-lang/crates.io-index".proc-macro2."1.0.43" { inherit profileName; };
      quote = rustPackages."registry+https://github.com/rust-lang/crates.io-index".quote."1.0.21" { inherit profileName; };
      syn = rustPackages."registry+https://github.com/rust-lang/crates.io-index".syn."1.0.99" { inherit profileName; };
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".serde_yaml."0.9.10" = overridableMkRustCrate (profileName: rec {
    name = "serde_yaml";
    version = "0.9.10";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "7a09f551ccc8210268ef848f0bab37b306e87b85b2e017b899e7fb815f5aed62"; };
    dependencies = {
      indexmap = rustPackages."registry+https://github.com/rust-lang/crates.io-index".indexmap."1.9.1" { inherit profileName; };
      itoa = rustPackages."registry+https://github.com/rust-lang/crates.io-index".itoa."1.0.3" { inherit profileName; };
      ryu = rustPackages."registry+https://github.com/rust-lang/crates.io-index".ryu."1.0.11" { inherit profileName; };
      serde = rustPackages."registry+https://github.com/rust-lang/crates.io-index".serde."1.0.144" { inherit profileName; };
      unsafe_libyaml = rustPackages."registry+https://github.com/rust-lang/crates.io-index".unsafe-libyaml."0.2.2" { inherit profileName; };
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".strsim."0.10.0" = overridableMkRustCrate (profileName: rec {
    name = "strsim";
    version = "0.10.0";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "73473c0e59e6d5812c5dfe2a064a6444949f089e20eec9a2e5506596494e4623"; };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".syn."1.0.99" = overridableMkRustCrate (profileName: rec {
    name = "syn";
    version = "1.0.99";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "58dbef6ec655055e20b86b15a8cc6d439cca19b667537ac6a1369572d151ab13"; };
    features = builtins.concatLists [
      [ "clone-impls" ]
      [ "default" ]
      [ "derive" ]
      [ "parsing" ]
      [ "printing" ]
      [ "proc-macro" ]
      [ "quote" ]
    ];
    dependencies = {
      proc_macro2 = rustPackages."registry+https://github.com/rust-lang/crates.io-index".proc-macro2."1.0.43" { inherit profileName; };
      quote = rustPackages."registry+https://github.com/rust-lang/crates.io-index".quote."1.0.21" { inherit profileName; };
      unicode_ident = rustPackages."registry+https://github.com/rust-lang/crates.io-index".unicode-ident."1.0.3" { inherit profileName; };
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".termcolor."1.1.3" = overridableMkRustCrate (profileName: rec {
    name = "termcolor";
    version = "1.1.3";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "bab24d30b911b2376f3a13cc2cd443142f0c81dda04c118693e35b3835757755"; };
    dependencies = {
      ${ if hostPlatform.isWindows then "winapi_util" else null } = rustPackages."registry+https://github.com/rust-lang/crates.io-index".winapi-util."0.1.5" { inherit profileName; };
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".textwrap."0.15.0" = overridableMkRustCrate (profileName: rec {
    name = "textwrap";
    version = "0.15.0";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "b1141d4d61095b28419e22cb0bbf02755f5e54e0526f97f1e3d1d160e60885fb"; };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".unicode-ident."1.0.3" = overridableMkRustCrate (profileName: rec {
    name = "unicode-ident";
    version = "1.0.3";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "c4f5b37a154999a8f3f98cc23a628d850e154479cd94decf3414696e12e31aaf"; };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".unsafe-libyaml."0.2.2" = overridableMkRustCrate (profileName: rec {
    name = "unsafe-libyaml";
    version = "0.2.2";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "931179334a56395bcf64ba5e0ff56781381c1a5832178280c7d7f91d1679aeb0"; };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".winapi."0.3.9" = overridableMkRustCrate (profileName: rec {
    name = "winapi";
    version = "0.3.9";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "5c839a674fcd7a98952e593242ea400abe93992746761e38641405d28b00f419"; };
    features = builtins.concatLists [
      [ "consoleapi" ]
      [ "errhandlingapi" ]
      [ "fileapi" ]
      [ "minwinbase" ]
      [ "minwindef" ]
      [ "processenv" ]
      [ "std" ]
      [ "winbase" ]
      [ "wincon" ]
      [ "winerror" ]
      [ "winnt" ]
    ];
    dependencies = {
      ${ if hostPlatform.config == "i686-pc-windows-gnu" then "winapi_i686_pc_windows_gnu" else null } = rustPackages."registry+https://github.com/rust-lang/crates.io-index".winapi-i686-pc-windows-gnu."0.4.0" { inherit profileName; };
      ${ if hostPlatform.config == "x86_64-pc-windows-gnu" then "winapi_x86_64_pc_windows_gnu" else null } = rustPackages."registry+https://github.com/rust-lang/crates.io-index".winapi-x86_64-pc-windows-gnu."0.4.0" { inherit profileName; };
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".winapi-i686-pc-windows-gnu."0.4.0" = overridableMkRustCrate (profileName: rec {
    name = "winapi-i686-pc-windows-gnu";
    version = "0.4.0";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "ac3b87c63620426dd9b991e5ce0329eff545bccbbb34f3be09ff6fb6ab51b7b6"; };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".winapi-util."0.1.5" = overridableMkRustCrate (profileName: rec {
    name = "winapi-util";
    version = "0.1.5";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "70ec6ce85bb158151cae5e5c87f95a8e97d2c0c4b001223f33a334e3ce5de178"; };
    dependencies = {
      ${ if hostPlatform.isWindows then "winapi" else null } = rustPackages."registry+https://github.com/rust-lang/crates.io-index".winapi."0.3.9" { inherit profileName; };
    };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".winapi-x86_64-pc-windows-gnu."0.4.0" = overridableMkRustCrate (profileName: rec {
    name = "winapi-x86_64-pc-windows-gnu";
    version = "0.4.0";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "712e227841d057c1ee1cd2fb22fa7e5a5461ae8e48fa2ca79ec42cfc1931183f"; };
  });
  
  "registry+https://github.com/rust-lang/crates.io-index".yaml-rust."0.4.5" = overridableMkRustCrate (profileName: rec {
    name = "yaml-rust";
    version = "0.4.5";
    registry = "registry+https://github.com/rust-lang/crates.io-index";
    src = fetchCratesIo { inherit name version; sha256 = "56c1936c4cc7a1c9ab21a1ebb602eb942ba868cbd44a99cb7cdc5892335e1c85"; };
    dependencies = {
      linked_hash_map = rustPackages."registry+https://github.com/rust-lang/crates.io-index".linked-hash-map."0.5.6" { inherit profileName; };
    };
  });
  
}
