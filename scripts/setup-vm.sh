#!/usr/bin/env bash
# Set up VM testing environment for ekaos-install

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# Detect OS
detect_os() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        OS=$ID
    else
        OS="unknown"
    fi
    log_info "Detected OS: $OS"
}

# Install QEMU based on OS
install_qemu() {
    log_step "Installing QEMU..."

    case "$OS" in
        nixos)
            log_info "On NixOS, add to your configuration.nix:"
            echo "  virtualisation.libvirtd.enable = true;"
            echo "  environment.systemPackages = with pkgs; [ qemu ];"
            log_warn "Please add the above and run: sudo nixos-rebuild switch"
            ;;
        ubuntu|debian)
            sudo apt-get update
            sudo apt-get install -y qemu-system-x86 qemu-utils
            ;;
        fedora|rhel|centos)
            sudo dnf install -y qemu-kvm qemu-img
            ;;
        arch)
            sudo pacman -S --noconfirm qemu-base
            ;;
        *)
            log_error "Unsupported OS: $OS"
            log_info "Please install QEMU manually"
            exit 1
            ;;
    esac

    log_info "QEMU installation complete (or instructions provided)"
}

# Check if QEMU is installed
check_qemu() {
    if command -v qemu-system-x86_64 &> /dev/null; then
        log_info "✓ QEMU is installed"
        qemu-system-x86_64 --version | head -n1
        return 0
    else
        log_warn "QEMU not found"
        return 1
    fi
}

# Download NixOS ISO (optional)
download_nixos_iso() {
    local iso_dir="$HOME/.local/share/ekaos-install"
    local iso_url="https://channels.nixos.org/nixos-unstable/latest-nixos-minimal-x86_64-linux.iso"
    local iso_file="$iso_dir/nixos-minimal.iso"

    log_step "NixOS ISO setup"

    if [ -f "$iso_file" ]; then
        log_info "NixOS ISO already downloaded: $iso_file"
        log_info "To use it, run: export NIXOS_ISO=$iso_file"
        return 0
    fi

    log_info "NixOS ISO not found at: $iso_file"
    echo -n "Download NixOS minimal ISO (~900MB)? [y/N] "
    read -r response

    if [[ "$response" =~ ^[Yy]$ ]]; then
        mkdir -p "$iso_dir"
        log_info "Downloading NixOS ISO..."
        log_info "URL: $iso_url"
        log_info "Destination: $iso_file"

        if command -v curl &> /dev/null; then
            curl -L -o "$iso_file" "$iso_url"
        elif command -v wget &> /dev/null; then
            wget -O "$iso_file" "$iso_url"
        else
            log_error "Neither curl nor wget found. Please install one of them."
            exit 1
        fi

        log_info "✓ Download complete"
        log_info "To use it, run: export NIXOS_ISO=$iso_file"
    else
        log_info "Skipping download. You can download manually from:"
        log_info "  $iso_url"
        log_info "Then set: export NIXOS_ISO=/path/to/iso"
    fi
}

# Check KVM support
check_kvm() {
    log_step "Checking KVM support..."

    if [ -e /dev/kvm ]; then
        log_info "✓ KVM is available"
        if [ -r /dev/kvm ] && [ -w /dev/kvm ]; then
            log_info "✓ KVM is accessible"
        else
            log_warn "KVM device exists but is not accessible"
            log_info "You may need to add your user to the 'kvm' group:"
            log_info "  sudo usermod -a -G kvm $USER"
            log_info "  Then log out and log back in"
        fi
    else
        log_warn "KVM not available (VM will run slower)"
        log_info "This is normal on non-Linux systems or virtual machines"
    fi
}

# Print usage instructions
print_usage() {
    log_step "VM Testing Setup Complete!"
    echo ""
    echo "Next steps:"
    echo "  1. Set the NixOS ISO path:"
    echo "     export NIXOS_ISO=/path/to/nixos.iso"
    echo ""
    echo "  2. Run a test VM:"
    echo "     ./scripts/run-vm.sh"
    echo ""
    echo "  3. Inside the VM, build and run ekaos-install:"
    echo "     - The project needs to be available in the VM"
    echo "     - Consider using a shared folder or building a package"
    echo ""
    echo "For automated testing, see:"
    echo "  tests/integration/"
    echo ""
}

# Main
main() {
    echo "================================="
    echo "Ekaos Install - VM Setup"
    echo "================================="
    echo ""

    detect_os

    if ! check_qemu; then
        log_warn "QEMU is not installed"
        echo -n "Install QEMU now? [y/N] "
        read -r response
        if [[ "$response" =~ ^[Yy]$ ]]; then
            install_qemu
        else
            log_info "Skipping QEMU installation"
        fi
    fi

    check_kvm
    download_nixos_iso
    print_usage
}

main "$@"
