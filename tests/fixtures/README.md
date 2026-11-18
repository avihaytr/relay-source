# Generated test fixtures

Both fixtures have explicit MPL-2.0 scope in [LICENSING.md](../../LICENSING.md); binary/strict-parser formats cannot carry inline SPDX comments.

`java-p256-proof.txt` contains only a generated public key, ECDSA signature and synthetic transcript. `IdentityProofTest --vector` generates the three-line format; signatures can differ between runs. No private key is included.

`keyframe.h264` is one synthetic solid-red 32×32 frame with no external media. Its 653-byte SHA-256 is `82d395953d01e24b07e7a2b698ecaf3beea09b869c47b7e22262fe9a9b34ede4`. It was reproduced with FFmpeg 7.1.5 and x264 core 165 r3222 b35605a:

```sh
ffmpeg -hide_banner -loglevel error \
  -f lavfi -i 'color=c=red:s=32x32:r=1:d=1,format=yuv420p' \
  -frames:v 1 -an -c:v libx264 -preset ultrafast -tune zerolatency \
  -profile:v baseline -level:v 1.0 -threads 1 \
  -x264-params 'keyint=1:min-keyint=1:scenecut=0:repeat-headers=1:threads=1' \
  -f h264 -y tests/fixtures/keyframe.h264
```

Preserve x264's embedded encoder identification/copyright metadata. Licensing an encoder does not automatically license all of its output; this fixture's explicit MPL grant applies to the extent rights exist. This output notice does not replace the separate license/source obligations for distributing FFmpeg or x264 themselves.
