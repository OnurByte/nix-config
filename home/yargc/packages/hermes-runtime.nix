{
  ghostty,
  libnotify,
  python3,
  writeShellApplication,
}:
writeShellApplication {
  name = "vesper-hermes";

  runtimeInputs = [
    ghostty
    libnotify
    python3
  ];

  text = ''
    exec python3 ${./hermes-runtime.py} "$@"
  '';
}
