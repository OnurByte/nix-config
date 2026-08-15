{
  config,
  lib,
  pkgs,
  ...
}:
{
  # Current CachyOS machine runs a very recent kernel; keep NixOS similarly fresh
  # without pulling in a third-party kernel patchset.
  boot.kernelPackages = pkgs.linuxPackages_latest;

  boot.kernelParams = [
    "amd_pstate=active"
    "nvidia.NVreg_TemporaryFilePath=/var/tmp"
  ];

  hardware.cpu.amd.updateMicrocode = lib.mkDefault config.hardware.enableRedistributableFirmware;

  # IdeaPad Gaming 3 16ARH7 / Ryzen 5 6600H / Rembrandt iGPU + RTX 3050 Mobile.
  # lspci:
  #   01:00.0 NVIDIA GA107M   -> PCI:1:0:0
  #   05:00.0 AMD Rembrandt  -> PCI:5:0:0
  services.xserver.videoDrivers = [
    "amdgpu"
    "nvidia"
  ];

  hardware.nvidia = {
    modesetting.enable = true;
    open = true;
    nvidiaSettings = true;

    powerManagement = {
      enable = true;
      finegrained = true;
    };

    prime = {
      offload.enable = true;
      offload.enableOffloadCmd = true;
      nvidiaBusId = "PCI:1:0:0";
      amdgpuBusId = "PCI:5:0:0";
    };
  };
}
