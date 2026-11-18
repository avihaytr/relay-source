// SPDX-License-Identifier: MPL-2.0
package app.relay.companion;

import android.content.Context;
import android.security.keystore.*;
import android.util.AtomicFile;
import org.json.*;
import java.io.*;
import java.security.*;
import java.security.spec.ECGenParameterSpec;
import javax.crypto.*;

/** Identity keys never leave Android Keystore. Only public trust records are saved. */
public final class DeviceTrust {
    private static final String IDENTITY="relay.device.identity.v1",MAC="relay.trust.integrity.v1";
    private static AtomicFile file;
    private static KeyStore keys;
    private static JSONObject data;
    private static long revision;
    public static synchronized void init(Context context)throws Exception{
        if(file!=null&&data!=null){ensureIdentity();return;}
        keys=KeyStore.getInstance("AndroidKeyStore");keys.load(null);ensureIdentity();
        if(!keys.containsAlias(MAC)){KeyGenerator gen=KeyGenerator.getInstance("HmacSHA256","AndroidKeyStore");gen.init(new KeyGenParameterSpec.Builder(MAC,KeyProperties.PURPOSE_SIGN|KeyProperties.PURPOSE_VERIFY).setDigests(KeyProperties.DIGEST_SHA256).build());gen.generateKey();}
        file=new AtomicFile(new File(context.getNoBackupFilesDir(),"relay-trusted-devices.json"));
        try {
        try(InputStream in=file.openRead()){
            ByteArrayOutputStream bytes=new ByteArrayOutputStream();byte[] buffer=new byte[1024];int n;while((n=in.read(buffer))!=-1){bytes.write(buffer,0,n);if(bytes.size()>128*1024)throw new IOException("Oversized trust store");}
            JSONObject envelope=new JSONObject(bytes.toString("UTF-8"));String payload=envelope.getString("payload");
            if(!MessageDigest.isEqual(mac(payload),decode(envelope.getString("mac"))))throw new IOException("Trust integrity failed");
            data=new JSONObject(payload);if(!publicKeyId().equals(data.optString("localIdentity")))throw new IOException("Identity changed");
            JSONObject peers=data.getJSONObject("peers");if(peers.length()>128)throw new IOException("Too many peers");
            java.util.Iterator<String> ids=peers.keys();while(ids.hasNext()){String id=ids.next();if(!id.matches("[0-9a-f]{64}"))throw new IOException("Invalid trusted identity");peers.getJSONObject(id).getString("name");}
        }catch(FileNotFoundException absent){data=empty(false);save();}
        catch(Exception invalid){data=empty(true);save();} // Fail closed: no inherited trust after corruption/key loss.
        } catch(Exception failure){data=null;file=null;revision++;throw failure;}
    }
    private static void ensureIdentity()throws Exception{
        if(!keys.containsAlias(IDENTITY)){KeyPairGenerator gen=KeyPairGenerator.getInstance("EC","AndroidKeyStore");gen.initialize(new KeyGenParameterSpec.Builder(IDENTITY,KeyProperties.PURPOSE_SIGN|KeyProperties.PURPOSE_VERIFY).setAlgorithmParameterSpec(new ECGenParameterSpec("secp256r1")).setDigests(KeyProperties.DIGEST_SHA256).build());gen.generateKeyPair();}
        if(data!=null&&!publicKeyId().equals(data.optString("localIdentity"))){data=empty(true);revision++;RelayConnection.disconnectAll();save();}
    }
    private static JSONObject empty(boolean require)throws Exception{return new JSONObject().put("localIdentity",publicKeyId()).put("requireApproval",require).put("peers",new JSONObject());}
    private static byte[] mac(String value)throws Exception{Mac mac=Mac.getInstance("HmacSHA256");mac.init(keys.getKey(MAC,null));return mac.doFinal(value.getBytes("UTF-8"));}
    private static void save()throws Exception{
        String payload=data.toString();byte[] bytes=new JSONObject().put("payload",payload).put("mac",hex(mac(payload))).toString().getBytes("UTF-8");
        TrustStorage.write(file,bytes);
    }
    public static String hex(byte[] bytes){return IdentityProof.hex(bytes);}
    private static byte[] decode(String value)throws Exception{if(!value.matches("[0-9a-f]{64}"))throw new IOException("Invalid integrity code");byte[] out=new byte[value.length()/2];for(int i=0;i<out.length;i++)out[i]=(byte)Integer.parseInt(value.substring(i*2,i*2+2),16);return out;}
    public static synchronized String publicKey()throws Exception{return hex(IdentityProof.rawPublicKey(keys.getCertificate(IDENTITY).getPublicKey()));}
    private static String publicKeyId()throws Exception{String value=publicKey();byte[] raw=new byte[65];for(int i=0;i<65;i++)raw[i]=(byte)Integer.parseInt(value.substring(i*2,i*2+2),16);return hex(MessageDigest.getInstance("SHA-256").digest(raw));}
    public static synchronized String sign(String transcript)throws Exception{Signature signature=Signature.getInstance("SHA256withECDSA");signature.initSign((PrivateKey)keys.getKey(IDENTITY,null));signature.update(transcript.getBytes("UTF-8"));return hex(signature.sign());}
    public static synchronized boolean trusted(String pin)throws Exception{ensureIdentity();return pin!=null&&pin.matches("[0-9a-f]{64}")&&data.getJSONObject("peers").has(pin);}
    public static synchronized long revision()throws Exception{ensureIdentity();if(data==null)throw new IOException("Trust store unavailable");return revision;}
    public static synchronized boolean requireApproval()throws Exception{ensureIdentity();return data.getBoolean("requireApproval");}
    public static synchronized void setApproval(boolean required)throws Exception{JSONObject before=data;data=new JSONObject(data.toString()).put("requireApproval",required);try{save();revision++;}catch(Exception e){data=before;throw e;}}
    public static synchronized void finish(String pin,String name,boolean remember,boolean wasTrusted,long expected)throws Exception{
        ensureIdentity();if(revision!=expected||(wasTrusted&&!trusted(pin)))throw new IOException("Trust changed during authentication");
        if(remember){JSONObject before=data;data=new JSONObject(data.toString());JSONObject peers=data.getJSONObject("peers");if(peers.length()>=128&&!peers.has(pin))throw new IOException("Trusted-device limit reached");peers.put(pin,new JSONObject().put("name",name==null?"Computer":name));try{save();revision++;}catch(Exception e){data=before;throw e;}}
    }
    public static synchronized JSONObject peers()throws Exception{ensureIdentity();return new JSONObject(data.getJSONObject("peers").toString());}
    public static synchronized void forget(String pin)throws Exception{JSONObject before=data;data=new JSONObject(data.toString());data.getJSONObject("peers").remove(pin);try{save();revision++;}catch(Exception e){data=before;throw e;}RelayConnection.disconnect(pin);}
    public static synchronized void resetIdentity()throws Exception{keys.deleteEntry(IDENTITY);ensureIdentity();data=empty(true);save();revision++;RelayConnection.disconnectAll();}
}
