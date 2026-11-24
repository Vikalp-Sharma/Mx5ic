import torch
import torch.nn as nn
import onnx

class CausalConv1d(nn.Module):
    """Causal 1D Convolution with left-padding to prevent looking into the future."""
    def __init__(self, in_channels, out_channels, kernel_size, dilation=1):
        super().__init__()
        self.padding = (kernel_size - 1) * dilation
        self.conv = nn.Conv1d(in_channels, out_channels, kernel_size, dilation=dilation)

    def forward(self, x):
        # x: [B, C, T]
        x_pad = nn.functional.pad(x, (self.padding, 0))
        return self.conv(x_pad)

class CausalAudioBlock(nn.Module):
    """Residual causal dilated convolution block with LeakyReLU."""
    def __init__(self, channels, kernel_size=3, dilation=1):
        super().__init__()
        self.conv1 = CausalConv1d(channels, channels, kernel_size, dilation)
        self.act = nn.LeakyReLU(0.2)
        self.conv2 = CausalConv1d(channels, channels, 1)

    def forward(self, x):
        res = x
        out = self.act(self.conv1(x))
        out = self.conv2(out)
        return res + out

class CausalAudioEnhancer(nn.Module):
    """
    Streaming causal neural audio enhancer (representing Bandwidth Extension & Tone Net).
    Operates chunk-by-chunk in real time without future lookahead.
    """
    def __init__(self, in_channels=1, hidden_channels=24, num_blocks=4):
        super().__init__()
        self.input_conv = CausalConv1d(in_channels, hidden_channels, kernel_size=3)
        self.blocks = nn.ModuleList([
            CausalAudioBlock(hidden_channels, kernel_size=3, dilation=2**i)
            for i in range(num_blocks)
        ])
        self.output_conv = CausalConv1d(hidden_channels, in_channels, kernel_size=3)
        self.tanh = nn.Tanh()

    def forward(self, x):
        # x: [Batch=1, Channels=1, Samples=128]
        feat = self.input_conv(x)
        for block in self.blocks:
            feat = block(feat)
        out = self.output_conv(feat)
        return self.tanh(x + 0.3 * out)

def main():
    print("[*] Instantiating Causal Audio Enhancer model...")
    model = CausalAudioEnhancer(in_channels=1, hidden_channels=24, num_blocks=4)
    model.eval()

    dummy_input = torch.randn(1, 1, 128, dtype=torch.float32)
    onnx_path = "spikes/spike_c_dummy_onnx/dummy_causal_enhancer.onnx"

    print(f"[*] Exporting model to ONNX: {onnx_path}...")
    torch.onnx.export(
        model,
        dummy_input,
        onnx_path,
        export_params=True,
        opset_version=17,
        do_constant_folding=True,
        input_names=["audio_in"],
        output_names=["audio_out"],
        dynamic_axes={"audio_in": {2: "samples"}, "audio_out": {2: "samples"}},
    )

    onnx_model = onnx.load(onnx_path)
    onnx.checker.check_model(onnx_model)
    file_size_kb = len(onnx_model.SerializeToString()) / 1024.0
    print(f"[+] ONNX model verified successfully! File size: {file_size_kb:.2f} KB")

if __name__ == "__main__":
    main()
