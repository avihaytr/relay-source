// SPDX-License-Identifier: MPL-2.0
package app.relay.companion;

import android.system.Os;
import android.util.AtomicFile;
import java.io.FileOutputStream;

/** Protect the actual write descriptor: AtomicFile may use a separate .new path. */
public final class TrustStorage {
    private TrustStorage() {}
    public static void write(AtomicFile file, byte[] bytes) throws Exception {
        FileOutputStream output = null;
        try {
            output = file.startWrite();
            Os.fchmod(output.getFD(), 0600);
            output.write(bytes);
            file.finishWrite(output);
        } catch (Exception failure) {
            if (output != null) file.failWrite(output);
            throw failure;
        }
    }
}
