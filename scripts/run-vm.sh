#!/usr/bin/env bash
# Run a test VM for ekaos-install development and testing

set -euo pipefail

# Configuration
VM_NAME="ekaos-test"
VM_DISK_SIZE="20G"
VM_RAM="2048"
VM_CPUS="2"
DISK_IMAGE="/tmp/${VM_NAME}.qcow2"
NIXOS_ISO="${NIXOS_ISO:-}"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
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

# Check for required tools
check_requirements() {
    if ! command -v qemu-system-x86_64 &> /dev/null; then
        log_error "qemu-system-x86_64 not found. Please install QEMU."
        exit 1
    fi

    if [ -z "$NIXOS_ISO" ]; then
        log_error "NIXOS_ISO environment variable not set."
        log_info "Please download NixOS ISO and set: export NIXOS_ISO=/path/to/nixos.iso"
        exit 1
    fi

    if [ ! -f "$NIXOS_ISO" ]; then
        log_error "NixOS ISO not found at: $NIXOS_ISO"
        exit 1
    fi
}

# Create disk image if it doesn't exist
create_disk() {
    if [ ! -f "$DISK_IMAGE" ]; then
        log_info "Creating disk image: $DISK_IMAGE"
        qemu-img create -f qcow2 "$DISK_IMAGE" "$VM_DISK_SIZE"
    else
        log_info "Using existing disk image: $DISK_IMAGE"
    fi
}

# Start the VM
start_vm() {
    log_info "Starting VM: $VM_NAME"
    log_info "  RAM: ${VM_RAM}MB"
    log_info "  CPUs: $VM_CPUS"
    log_info "  Disk: $DISK_IMAGE"
    log_info "  ISO: $NIXOS_ISO"
    log_info ""
    log_info "VM will boot from NixOS ISO"
    log_info "Press Ctrl+Alt+G to release mouse/keyboard"
    log_info "Press Ctrl+C in this terminal to shut down VM"
    log_info ""

    qemu-system-x86_64 \
        -name "$VM_NAME" \
        -m "$VM_RAM" \
        -smp "$VM_CPUS" \
        -enable-kvm \
        -cpu host \
        -drive file="$DISK_IMAGE",format=qcow2,if=virtio \
        -cdrom "$NIXOS_ISO" \
        -boot d \
        -net nic,model=virtio \
        -net user \
        -vga virtio \
        -display gtk
}

# Clean up disk image
cleanup() {
    if [ -f "$DISK_IMAGE" ]; then
        log_warn "Removing disk image: $DISK_IMAGE"
        rm -f "$DISK_IMAGE"
    fi
}

# Main
main() {
    case "${1:-}" in
        clean)
            cleanup
            log_info "Cleanup complete"
            ;;
        *)
            check_requirements
            create_disk
            start_vm
            ;;
    esac
}

main "$@"
