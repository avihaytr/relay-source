// SPDX-License-Identifier: MPL-2.0
package app.relay.companion;
import java.math.BigInteger;
import java.security.PublicKey;
import java.security.interfaces.ECPublicKey;
import java.nio.charset.StandardCharsets;
import java.security.MessageDigest;
import java.nio.ByteBuffer;
/** Canonical, domain-separated protocol transcript shared by Android and host tests. */
public final class IdentityProof {
    private IdentityProof(){}
    public static String hex(byte[] bytes){StringBuilder out=new StringBuilder();for(byte b:bytes)out.append(String.format("%02x",b&255));return out.toString();}
    private static void coordinate(BigInteger value,byte[] out,int offset){byte[] bytes=value.toByteArray();int count=Math.min(32,bytes.length);System.arraycopy(bytes,bytes.length-count,out,offset+32-count,count);}
    public static byte[] rawPublicKey(PublicKey publicKey){ECPublicKey key=(ECPublicKey)publicKey;byte[] point=new byte[65];point[0]=4;coordinate(key.getW().getAffineX(),point,1);coordinate(key.getW().getAffineY(),point,33);return point;}
    public static String transcript(String pin,String nonce,String publicKey,String mode,int quality,String name,String auth,String code,boolean remember){return "Relay-v6-trust1\n"+pin+"\n"+nonce+"\n"+publicKey+"\n"+mode+"\n"+quality+"\n"+hex(name.getBytes(StandardCharsets.UTF_8))+"\n"+auth+"\n"+code+"\n"+(remember?"1":"0")+"\n";}
    public static String verificationCode(String pin,String nonce,String publicKey)throws Exception{byte[] digest=MessageDigest.getInstance("SHA-256").digest(("Relay-v6-pin-check\n"+pin+"\n"+nonce+"\n"+publicKey).getBytes(StandardCharsets.UTF_8));long value=Integer.toUnsignedLong(ByteBuffer.wrap(digest).getInt())%1_000_000L;return String.format(java.util.Locale.ROOT,"%06d",value);}
}
