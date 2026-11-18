// SPDX-License-Identifier: MPL-2.0
package app.relay.companion;
import android.net.Uri;
import java.net.*;
import java.io.*;
import org.json.JSONObject;
import java.security.*;
import java.security.cert.*;
import javax.net.ssl.*;

/** Pinned TLS and signed device proof; mDNS is only a locator. */
public final class RelayConnection {
    private RelayConnection(){}
    private static final java.util.concurrent.ConcurrentHashMap<SSLSocket,String> active=new java.util.concurrent.ConcurrentHashMap<>();
    public static void configure(android.content.Context context)throws Exception{DeviceTrust.init(context.getApplicationContext());}
    public static void disconnect(String pin){for(java.util.Map.Entry<SSLSocket,String> entry:active.entrySet())if(pin.equals(entry.getValue())){try{entry.getKey().close();}catch(Exception ignored){}active.remove(entry.getKey());}}
    public static void disconnectAll(){for(SSLSocket socket:active.keySet())try{socket.close();}catch(Exception ignored){}active.clear();}
    public static boolean validTrusted(Uri target){try{return "relay-trusted".equals(target.getScheme())&&target.getHost()!=null&&target.getPort()>0&&target.getPort()<=65535&&target.getPathSegments().size()==1&&"6".equals(target.getQueryParameter("v"))&&target.getUserInfo()==null&&target.getFragment()==null&&DeviceTrust.trusted(target.getPathSegments().get(0));}catch(Exception error){return false;}}
    public static boolean validPin(Uri target){try{return "relay-pin".equals(target.getScheme())&&target.getHost()!=null&&target.getPort()>0&&target.getPort()<=65535&&target.getPathSegments().size()==1&&target.getPathSegments().get(0).matches("[0-9]{6}")&&"6".equals(target.getQueryParameter("v"))&&target.getUserInfo()==null&&target.getFragment()==null&&(target.getQueryParameter("q")==null||target.getQueryParameter("q").matches("[012]"));}catch(Exception error){return false;}}
    public static boolean validTarget(Uri target){return validQr(target)||validTrusted(target)||validPin(target);}

    /** Enrolls a new phone without starting a screen/audio session. */
    public static String pair(Uri target,String deviceName,java.util.function.Consumer<SSLSocket> onSocket)throws Exception{
        return pair(target,deviceName,onSocket,null);
    }
    public static String pair(Uri target,String deviceName,java.util.function.Consumer<SSLSocket> onSocket,java.util.function.Function<String,Boolean> verifyCode)throws Exception{try(SSLSocket socket=openInternal(target,"pair",deviceName,onSocket,verifyCode)){byte[] digest=MessageDigest.getInstance("SHA-256").digest(socket.getSession().getPeerCertificates()[0].getEncoded());StringBuilder pin=new StringBuilder();for(byte b:digest)pin.append(String.format(java.util.Locale.ROOT,"%02x",b&255));return pin.toString();} }

    public static boolean validQr(Uri target){return "relay".equals(target.getScheme())&&target.getHost()!=null&&target.getPort()>0&&target.getPort()<=65535&&target.getPathSegments().size()==2&&target.getPathSegments().get(0).matches("[0-9a-f]{64}")&&target.getPathSegments().get(1).matches("[0-9a-f]{64}")&&"6".equals(target.getQueryParameter("v"))&&target.getUserInfo()==null&&target.getFragment()==null&&(target.getQueryParameter("q")==null||target.getQueryParameter("q").matches("[012]"));}
    private static void writeJson(DataOutputStream output,JSONObject value)throws Exception{byte[] bytes=value.toString().getBytes("UTF-8");if(bytes.length>1024)throw new IOException("Invalid authentication request");output.writeInt(bytes.length);output.write(bytes);output.flush();}
    private static JSONObject readJson(DataInputStream input)throws Exception{int size=input.readInt();if(size<1||size>1024)throw new IOException("Pairing rejected. Scan a fresh QR code and approve on the PC.");byte[] bytes=new byte[size];input.readFully(bytes);return new JSONObject(new String(bytes,"UTF-8"));}
    public static SSLSocket open(Uri target,String mode,String deviceName,java.util.function.Consumer<SSLSocket> onSocket)throws Exception{return openInternal(target,mode,deviceName,onSocket,null);}
    private static SSLSocket openInternal(Uri target,String mode,String deviceName,java.util.function.Consumer<SSLSocket> onSocket,java.util.function.Function<String,Boolean> verifyCode)throws Exception{
        boolean remembered=validTrusted(target);
        boolean pairing=mode.equals("pair");
        boolean pinPair=validPin(target);
        if(!validTarget(target)||!(pairing||mode.equals("send")||mode.equals("view"))||(pairing&&validTrusted(target))||(pinPair&&(!pairing||verifyCode==null)))throw new IOException("Scan the current pairing QR code or enter the current PC PIN");
        String code=remembered?"":target.getPathSegments().get(0);
        final String expectedPin=remembered?target.getPathSegments().get(0):validQr(target)?target.getPathSegments().get(1):null;
        final java.util.concurrent.atomic.AtomicReference<String> actualPin=new java.util.concurrent.atomic.AtomicReference<>();
        long trustRevision=DeviceTrust.revision();String publicKey=DeviceTrust.publicKey();boolean remember=pairing||!remembered&&"1".equals(target.getQueryParameter("remember"));
        SSLContext context=SSLContext.getInstance("TLS");context.init(null,new TrustManager[]{new X509TrustManager(){
            public X509Certificate[] getAcceptedIssuers(){return new X509Certificate[0];}
            public void checkClientTrusted(X509Certificate[] chain,String type)throws CertificateException{throw new CertificateException("Not a client verifier");}
            public void checkServerTrusted(X509Certificate[] chain,String type)throws CertificateException{
                try{if(chain.length==0)throw new CertificateException("No computer certificate");byte[] digest=MessageDigest.getInstance("SHA-256").digest(chain[0].getEncoded());StringBuilder actual=new StringBuilder();for(byte b:digest)actual.append(String.format(java.util.Locale.ROOT,"%02x",b&255));String fingerprint=actual.toString();if(expectedPin!=null&&!MessageDigest.isEqual(expectedPin.getBytes("UTF-8"),fingerprint.getBytes("UTF-8")))throw new CertificateException("Computer identity does not match the scanned QR");actualPin.compareAndSet(null,fingerprint);}
                catch(Exception error){throw new CertificateException("Computer identity verification failed");}
            }
        }},new SecureRandom());
        SSLSocket socket=(SSLSocket)context.getSocketFactory().createSocket();
        try{
            active.entrySet().removeIf(entry->entry.getKey().isClosed());if(expectedPin!=null){active.put(socket,expectedPin);onSocket.accept(socket);}
            socket.connect(new InetSocketAddress(target.getHost(),target.getPort()),6000);socket.setSoTimeout(60000);socket.setTcpNoDelay(true);socket.startHandshake();
            String pin=actualPin.get();if(pin==null)throw new IOException("PC certificate unavailable");if(expectedPin==null){active.put(socket,pin);onSocket.accept(socket);}
            DataOutputStream output=new DataOutputStream(socket.getOutputStream());DataInputStream input=new DataInputStream(socket.getInputStream());
            int quality=Math.max(0,Math.min(2,Integer.parseInt(target.getQueryParameter("q")==null?"2":target.getQueryParameter("q"))));
            String auth=remembered?"trusted":pinPair?"pin":"qr";
            output.writeBytes("RLY6");writeJson(output,new JSONObject().put("version",6).put("code",code).put("name",deviceName).put("mode",mode).put("quality",quality).put("public_key",publicKey).put("auth",auth).put("remember",remember));
            JSONObject challenge=readJson(input);String nonce=challenge.optString("nonce");if(!"challenge".equals(challenge.optString("status"))||!nonce.matches("[0-9a-f]{64}"))throw new IOException("Device challenge missing");
            if(pinPair){String expected=IdentityProof.verificationCode(pin,nonce,publicKey);String received=challenge.optString("verifyCode");if(!MessageDigest.isEqual(expected.getBytes("UTF-8"),received.getBytes("UTF-8"))||!Boolean.TRUE.equals(verifyCode.apply(expected)))throw new IOException("Verification code was not confirmed");}
        String transcript=IdentityProof.transcript(pin,nonce,publicKey,mode,quality,deviceName,auth,code,remember);
            writeJson(output,new JSONObject().put("signature",DeviceTrust.sign(transcript)));
            JSONObject approved=readJson(input);String session=approved.optString("session");if(!"approved".equals(approved.optString("status"))||!session.matches("[0-9a-f]{64}"))throw new IOException("Pairing not approved. Scan again and approve on the PC.");
            writeJson(output,new JSONObject().put("session",session));if(input.read()!='O'||input.read()!='K')throw new IOException("Session authorization expired or rejected");
            DeviceTrust.finish(pin,target.getQueryParameter("computer"),remember,remembered,trustRevision);
            socket.setSoTimeout(150000);return socket;
        }catch(Exception error){active.remove(socket);socket.close();throw new IOException("Trust could not be verified. Check PC approval, or scan a fresh QR to pair again.");}
    }
}
