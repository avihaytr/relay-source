// SPDX-License-Identifier: MPL-2.0
import app.relay.companion.IdentityProof;
import java.security.*;
import java.security.spec.ECGenParameterSpec;
import java.nio.charset.StandardCharsets;
public final class IdentityProofTest {
    public static void main(String[] args)throws Exception{
        KeyPairGenerator gen=KeyPairGenerator.getInstance("EC");gen.initialize(new ECGenParameterSpec("secp256r1"));KeyPair pair=gen.generateKeyPair();
        String publicKey=IdentityProof.hex(IdentityProof.rawPublicKey(pair.getPublic()));if(publicKey.length()!=130||!publicKey.startsWith("04"))throw new AssertionError("Uncompressed P-256 key encoding");
        String pin=new String(new char[64]).replace('\0','1'),nonce=new String(new char[64]).replace('\0','2');
        if(!IdentityProof.verificationCode(pin,nonce,"04"+new String(new char[128]).replace('\0','a')).equals("766134"))throw new AssertionError("PIN pairing verification code must match the Rust protocol");
        String message=IdentityProof.transcript(pin,nonce,publicKey,"view",2,"Test phone","trusted","",false);
        Signature signer=Signature.getInstance("SHA256withECDSA");signer.initSign(pair.getPrivate());signer.update(message.getBytes(StandardCharsets.UTF_8));byte[] signature=signer.sign();
        Signature verifier=Signature.getInstance("SHA256withECDSA");verifier.initVerify(pair.getPublic());verifier.update(message.getBytes(StandardCharsets.UTF_8));if(!verifier.verify(signature))throw new AssertionError("Signature must verify");
        verifier.initVerify(pair.getPublic());verifier.update(IdentityProof.transcript(pin,nonce,publicKey,"send",2,"Test phone","trusted","",false).getBytes(StandardCharsets.UTF_8));if(verifier.verify(signature))throw new AssertionError("Direction must be signed");
        verifier.initVerify(gen.generateKeyPair().getPublic());verifier.update(message.getBytes(StandardCharsets.UTF_8));if(verifier.verify(signature))throw new AssertionError("Impersonation must fail");
        if(args.length>0&&args[0].equals("--vector")){System.out.println(publicKey);System.out.println(IdentityProof.hex(signature));System.out.println(IdentityProof.hex(message.getBytes(StandardCharsets.UTF_8)));}
        else System.out.println("PASS Java identity encoding, signing, direction binding and impersonation checks");
    }
}
