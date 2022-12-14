{ lib
, stdenv
, fetchFromGitHub
, boost
, clang-tools
, cmake
, curl
, gtest
, llvmPackages
, log4cxx
, openssl
, pkg-config
, protobuf
, snappy
, zlib
, zstd
, snappySupport ? false
, zstdSupport ? true
, log4cxxSupport ? false
, buildTest ? false
}:
let
  /*
    Check if null or false
    Example:
    let result = enableFeature null
    => "OFF"
    let result = enableFeature false
    => "OFF"
    let result = enableFeature «derivation»
    => "ON"
  */
  enableCmakeFeature = p: if (p == null || p == false) then "OFF" else "ON";
  clangTools = clang-tools.override { inherit stdenv llvmPackages; };
in
stdenv.mkDerivation rec {
  pname = "libpulsar";
  version = "3.1.0";

  src = fetchFromGitHub {
    owner = "apache";
    repo = "pulsar-client-cpp";
    rev = "v${version}";
    hash = "sha256-/zU3otmlGhFvvgfC2myyRh0b0kOohzipjI09XjFmB2g=";
  };

  patches = [
    ./auxv.patch
  ];

  # clang-tools needed for clang-format
  nativeBuildInputs = [ cmake pkg-config clangTools ];

  buildInputs = [ boost curl openssl protobuf zlib ]
    ++ lib.optional snappySupport snappy
    ++ lib.optional zstdSupport zstd
    ++ lib.optional log4cxxSupport log4cxx
    ++ lib.optional buildTest gtest;

  CLANG_TOOLS_PATH = "${clang-tools}/bin";

  # Needed for GCC on Linux
  NIX_CFLAGS_COMPILE = [ "-Wno-error=return-type" ];

  cmakeFlags = [
    "-DUSE_LOG4CXX=${enableCmakeFeature log4cxxSupport}"
    "-DBUILD_TESTS=${enableCmakeFeature buildTest}"
    "-DClangTools_PATH=${clangTools}/bin"
  ];

  enableParallelBuilding = true;
  doInstallCheck = true;
  installCheckPhase = ''
    echo ${lib.escapeShellArg ''
      #include <pulsar/Client.h>
      int main (int argc, char **argv) {
        pulsar::Client client("pulsar://localhost:6650");
        return 0;
      }
    ''} > test.cc
    $CXX test.cc -L $out/lib -I $out/include -lpulsar -o test
  '';

  meta = with lib; {
    homepage = "https://pulsar.apache.org/docs/en/client-libraries-cpp";
    description = "Apache Pulsar C++ library";

    platforms = platforms.all;
    license = licenses.asl20;
    maintainers = [ maintainers.fstnetowrk ];
  };
}
