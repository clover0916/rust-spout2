// DirectX-only bridge: no OpenGL context and no GPU -> CPU readback.
#include "SpoutDX.h"
#include <new>
#include <memory>

namespace {
struct Sender {
    ID3D11Device* device;
    spoutDX spout;

    explicit Sender(ID3D11Device* value) : device(value) { device->AddRef(); }
    ~Sender() {
        spout.ReleaseSender();
        spout.CloseDirectX11();
        device->Release();
    }
};
}

extern "C" void* rust_spout_dx_create(void* device, const char* name) noexcept {
    if (!device || !name || !name[0]) return nullptr;
    try {
        auto sender = std::unique_ptr<Sender>(new Sender(static_cast<ID3D11Device*>(device)));
        if (!sender->spout.OpenDirectX11(sender->device) || !sender->spout.SetSenderName(name))
            return nullptr;
        return sender.release();
    } catch (...) { return nullptr; }
}

extern "C" bool rust_spout_dx_send(void* handle, void* texture) noexcept {
    if (!handle || !texture) return false;
    try {
        auto* sender = static_cast<Sender*>(handle);
        auto* source = static_cast<ID3D11Texture2D*>(texture);
        ID3D11Device* sourceDevice = nullptr;
        source->GetDevice(&sourceDevice);
        const bool sameDevice = sourceDevice == sender->device;
        if (sourceDevice) sourceDevice->Release();
        if (!sameDevice) return false;
        D3D11_TEXTURE2D_DESC desc = {};
        source->GetDesc(&desc);
        // CopyResource requires matching texture shape. Spout's shared output
        // is one mip, one array slice, single-sampled, in a supported RGB format.
        if (!desc.Width || !desc.Height || desc.MipLevels != 1 || desc.ArraySize != 1
            || desc.SampleDesc.Count != 1 || desc.SampleDesc.Quality != 0
            || (desc.Format != DXGI_FORMAT_B8G8R8A8_UNORM
                && desc.Format != DXGI_FORMAT_R8G8B8A8_UNORM)) return false;
        return sender->spout.SendTexture(source);
    } catch (...) { return false; }
}

extern "C" void rust_spout_dx_release_sender(void* handle, const char* name) noexcept {
    if (handle) {
        auto* sender = static_cast<Sender*>(handle);
        sender->spout.ReleaseSender();
        // Upstream ReleaseSender clears the name. Preserve the configured name
        // for a later submission without keeping a discovery registration alive.
        sender->spout.SetSenderName(name);
    }
}

extern "C" void rust_spout_dx_destroy(void* handle) noexcept {
    delete static_cast<Sender*>(handle);
}
