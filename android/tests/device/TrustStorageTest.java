// SPDX-License-Identifier: MPL-2.0
package app.relay.companion.tests;

import android.app.Instrumentation;
import android.app.Activity;
import android.os.Bundle;
import android.system.Os;
import android.util.AtomicFile;
import app.relay.companion.DeviceTrust;
import app.relay.companion.TrustStorage;
import java.io.File;
import java.nio.charset.StandardCharsets;
import java.security.KeyStore;
import java.security.Signature;
import java.util.Arrays;

/** Local instrumentation only; never packaged into the production app. */
public final class TrustStorageTest extends Instrumentation {
    @Override public void onCreate(Bundle arguments) { super.onCreate(arguments); start(); }
    private static void check(boolean ok, String failure) {
        if (!ok) throw new AssertionError(failure);
    }
    private static byte[] decode(String hex) {
        byte[] bytes = new byte[hex.length()/2];
        for (int i=0;i<bytes.length;i++) bytes[i]=(byte)Integer.parseInt(hex.substring(2*i,2*i+2),16);
        return bytes;
    }
    @Override public void onStart() {
        Bundle result = new Bundle();
        File scratch = new File(getTargetContext().getCacheDir(), "trust-write-regression");
        AtomicFile atomic = new AtomicFile(scratch);
        try {
            atomic.delete();
            byte[] first = "initial trust write".getBytes(StandardCharsets.UTF_8);
            TrustStorage.write(atomic, first);
            check(Arrays.equals(atomic.readFully(), first), "Initial atomic write failed");
            check((Os.stat(scratch.getPath()).st_mode & 0777)==0600, "Initial file permissions");
            byte[] next = "replacement trust write".getBytes(StandardCharsets.UTF_8);
            TrustStorage.write(atomic, next);
            check(Arrays.equals(atomic.readFully(), next), "Replacement atomic write failed");
            check((Os.stat(scratch.getPath()).st_mode & 0777)==0600, "Replacement file permissions");
            DeviceTrust.init(getTargetContext());
            String publicKey = DeviceTrust.publicKey();
            boolean approval = DeviceTrust.requireApproval();
            String peers = DeviceTrust.peers().toString();
            DeviceTrust.setApproval(approval); // Exercise authenticated store persistence without changing policy.
            for (String field : new String[]{"data", "file"}) {
                java.lang.reflect.Field state = DeviceTrust.class.getDeclaredField(field);
                state.setAccessible(true); state.set(null, null);
            }
            DeviceTrust.init(getTargetContext());
            check(DeviceTrust.publicKey().equals(publicKey), "Identity changed on reload");
            check(DeviceTrust.requireApproval()==approval, "Approval policy changed on reload");
            check(DeviceTrust.peers().toString().equals(peers), "Peers changed on reload");
            File store = new File(getTargetContext().getNoBackupFilesDir(), "relay-trusted-devices.json");
            check((Os.stat(store.getPath()).st_mode & 0777)==0600, "Trust store permissions");
            byte[] message = "Relay on-device signing regression".getBytes(StandardCharsets.UTF_8);
            byte[] signed = decode(DeviceTrust.sign(new String(message,StandardCharsets.UTF_8)));
            KeyStore keys = KeyStore.getInstance("AndroidKeyStore"); keys.load(null);
            Signature verifier = Signature.getInstance("SHA256withECDSA");
            verifier.initVerify(keys.getCertificate("relay.device.identity.v1").getPublicKey());
            verifier.update(message); check(verifier.verify(signed), "AndroidKeyStore signature rejected");
            check(keys.getKey("relay.device.identity.v1",null).getEncoded()==null, "Private key must be non-exportable");
            result.putString("stream", "PASS: first/replacement atomic writes, 0600 permissions, HMAC-store reload, stable identity, AndroidKeyStore signing and non-exportable key\n");
            finish(Activity.RESULT_OK, result);
        } catch (Throwable failure) {
            result.putString("stream", "FAIL: " + failure.getClass().getSimpleName() + "\n");
            finish(Activity.RESULT_CANCELED, result);
        } finally { atomic.delete(); }
    }
}
