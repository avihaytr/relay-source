// SPDX-License-Identifier: MPL-2.0
package app.relay.companion;

import android.Manifest;
import android.app.Activity;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.hardware.Camera;
import android.os.*;
import android.view.*;
import android.widget.*;
import com.google.zxing.*;
import com.google.zxing.common.HybridBinarizer;
import java.util.*;
import java.util.concurrent.*;
import java.util.concurrent.atomic.AtomicBoolean;

/** Camera is active only on this screen; QR decoding is bounded to one worker. */
@SuppressWarnings("deprecation")
public class ScanActivity extends Activity implements SurfaceHolder.Callback {
    private SurfaceView preview;
    private Camera camera;
    private FrameLayout cameraStage;
    private int previewWidth,previewHeight,orientation=90;
    private int dp(int value){return (int)(getResources().getDisplayMetrics().density*value);}
    private TextView text(String value,int size,int color){TextView view=new TextView(this);view.setText(value);view.setTextSize(size);view.setTextColor(color);return view;}
    private void fitPreview(){if(previewWidth==0||cameraStage.getWidth()==0)return;boolean swapped=orientation%180!=0;float width=swapped?previewHeight:previewWidth,height=swapped?previewWidth:previewHeight;float scale=Math.max(cameraStage.getWidth()/width,cameraStage.getHeight()/height);FrameLayout.LayoutParams layout=new FrameLayout.LayoutParams(Math.round(width*scale),Math.round(height*scale),Gravity.CENTER);preview.setLayoutParams(layout);}

    private final AtomicBoolean busy=new AtomicBoolean();
    private final ExecutorService decoder=Executors.newSingleThreadExecutor();
    private boolean found;
    private long lastScan;
    @Override public void onCreate(Bundle state){
        super.onCreate(state);
        getWindow().setStatusBarColor(0xff101b30);getWindow().setNavigationBarColor(0xff101b30);getWindow().getDecorView().setSystemUiVisibility(0);
        LinearLayout box=new LinearLayout(this);box.setOrientation(LinearLayout.VERTICAL);box.setBackgroundColor(0xff101b30);
        if(Build.VERSION.SDK_INT>=30)box.setOnApplyWindowInsetsListener((view,insets)->{android.graphics.Insets bars=insets.getInsets(WindowInsets.Type.systemBars()|WindowInsets.Type.displayCutout());view.setPadding(bars.left,bars.top,bars.right,bars.bottom);return insets;});
        LinearLayout bar=new LinearLayout(this);bar.setGravity(Gravity.CENTER_VERTICAL);bar.setPadding(dp(12),dp(12),dp(20),dp(12));
        TextView back=text("‹",32,0xffffffff);back.setGravity(Gravity.CENTER);back.setContentDescription("Close QR scanner");back.setOnClickListener(v->finish());bar.addView(back,new LinearLayout.LayoutParams(dp(48),dp(48)));
        TextView title=text("Scan QR code",21,0xffffffff);title.setTypeface(null,1);bar.addView(title);box.addView(bar);
        TextView help=text("Open the connection QR code in Relay on your PC.",14,0xffc1cde0);help.setPadding(dp(24),0,dp(24),dp(20));box.addView(help);
        cameraStage=new FrameLayout(this);cameraStage.setClipChildren(true);cameraStage.setBackgroundColor(0xff090f1c);box.addView(cameraStage,new LinearLayout.LayoutParams(-1,0,1));
        preview=new SurfaceView(this);cameraStage.addView(preview,new FrameLayout.LayoutParams(-1,-1,Gravity.CENTER));preview.getHolder().addCallback(this);
        cameraStage.addOnLayoutChangeListener((v,l,t,r,b,ol,ot,or,ob)->{if(r-l!=or-ol||b-t!=ob-ot)fitPreview();});cameraStage.addView(new ScanFrame(),new FrameLayout.LayoutParams(-1,-1));
        LinearLayout footer=new LinearLayout(this);footer.setOrientation(LinearLayout.VERTICAL);footer.setPadding(dp(24),dp(22),dp(24),dp(20));TextView instruction=text("Place the QR code inside the frame",16,0xffffffff);instruction.setGravity(Gravity.CENTER);footer.addView(instruction);TextView detail=text("Relay connects automatically when it recognizes the code.",13,0xffaebed6);detail.setGravity(Gravity.CENTER);detail.setPadding(0,dp(8),0,dp(18));footer.addView(detail);
        Button cancel=new Button(this);cancel.setText("Cancel scanning");cancel.setAllCaps(false);cancel.setTextSize(15);cancel.setTextColor(0xffffffff);android.graphics.drawable.GradientDrawable background=new android.graphics.drawable.GradientDrawable();background.setColor(0xff24334d);background.setCornerRadius(dp(12));cancel.setBackground(new android.graphics.drawable.RippleDrawable(android.content.res.ColorStateList.valueOf(0x33ffffff),background,null));cancel.setOnClickListener(v->finish());footer.addView(cancel,new LinearLayout.LayoutParams(-1,dp(50)));box.addView(footer);setContentView(box);
        if(Build.VERSION.SDK_INT>=30)getWindow().getInsetsController().setSystemBarsAppearance(0,WindowInsetsController.APPEARANCE_LIGHT_STATUS_BARS|WindowInsetsController.APPEARANCE_LIGHT_NAVIGATION_BARS);
        if(checkSelfPermission(Manifest.permission.CAMERA)!=PackageManager.PERMISSION_GRANTED)requestPermissions(new String[]{Manifest.permission.CAMERA},7);
    }
    @Override public void onRequestPermissionsResult(int request,String[] permissions,int[] results){super.onRequestPermissionsResult(request,permissions,results);if(request==7){if(results.length>0&&results[0]==PackageManager.PERMISSION_GRANTED)open();else{Toast.makeText(this,"Camera permission is needed only for QR scanning. Nearby discovery works without it.",1).show();finish();}}}
    @Override public void surfaceCreated(SurfaceHolder holder){open();}
    @Override public void surfaceChanged(SurfaceHolder h,int f,int w,int height){}
    @Override public void surfaceDestroyed(SurfaceHolder holder){close();}
    @Override protected void onPause(){super.onPause();close();}
    @Override protected void onResume(){super.onResume();if(preview.getHolder().getSurface().isValid())open();}
    private void open(){
        if(camera!=null||found||checkSelfPermission(Manifest.permission.CAMERA)!=PackageManager.PERMISSION_GRANTED||!preview.getHolder().getSurface().isValid())return;
        try{
            int cameraId=0;Camera.CameraInfo info=new Camera.CameraInfo();for(int id=0;id<Camera.getNumberOfCameras();id++){Camera.getCameraInfo(id,info);if(info.facing==Camera.CameraInfo.CAMERA_FACING_BACK){cameraId=id;break;}}Camera.getCameraInfo(cameraId,info);int rotation=getWindowManager().getDefaultDisplay().getRotation()*90;orientation=(info.orientation-rotation+360)%360;
            camera=Camera.open(cameraId);Camera.Parameters p=camera.getParameters();Camera.Size size=null;
            for(Camera.Size candidate:p.getSupportedPreviewSizes())if(candidate.width<=1280&&(size==null||candidate.width>size.width))size=candidate;
            if(size!=null)p.setPreviewSize(size.width,size.height);
            if(p.getSupportedFocusModes().contains(Camera.Parameters.FOCUS_MODE_CONTINUOUS_PICTURE))p.setFocusMode(Camera.Parameters.FOCUS_MODE_CONTINUOUS_PICTURE);
            p.setPreviewFormat(android.graphics.ImageFormat.NV21);camera.setParameters(p);camera.setDisplayOrientation(orientation);camera.setPreviewDisplay(preview.getHolder());
            final Camera.Size actual=camera.getParameters().getPreviewSize();previewWidth=actual.width;previewHeight=actual.height;fitPreview();
            camera.addCallbackBuffer(new byte[actual.width*actual.height*3/2]);camera.addCallbackBuffer(new byte[actual.width*actual.height*3/2]);
            camera.setPreviewCallbackWithBuffer((bytes,source)->{
                long now=SystemClock.elapsedRealtime();
                if(found||now-lastScan<150||!busy.compareAndSet(false,true)){source.addCallbackBuffer(bytes);return;}lastScan=now;
                decoder.execute(()->{
                    try{
                        PlanarYUVLuminanceSource luminance=new PlanarYUVLuminanceSource(bytes,actual.width,actual.height,0,0,actual.width,actual.height,false);
                        Map<DecodeHintType,Object> hints=new EnumMap<>(DecodeHintType.class);hints.put(DecodeHintType.POSSIBLE_FORMATS,Collections.singletonList(BarcodeFormat.QR_CODE));
                        Result result=new MultiFormatReader().decode(new BinaryBitmap(new HybridBinarizer(luminance)),hints);
                        String value=result.getText();if(RelayConnection.validQr(android.net.Uri.parse(value)))runOnUiThread(()->{if(!found){found=true;setResult(RESULT_OK,new Intent().putExtra("connection",value));finish();}});
                    }catch(Exception ignored){}finally{busy.set(false);runOnUiThread(()->{try{if(camera==source&&!found)source.addCallbackBuffer(bytes);}catch(Exception ignored){}});}
                });
            });camera.startPreview();
        }catch(Exception e){Toast.makeText(this,"Could not open camera: "+e.getMessage(),1).show();finish();}
    }
    private class ScanFrame extends View {
        ScanFrame(){super(ScanActivity.this);setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);}
        @Override protected void onDraw(android.graphics.Canvas canvas){
            float size=Math.min(getWidth()-dp(48),getHeight()-dp(48));float x=(getWidth()-size)/2,y=(getHeight()-size)/2;android.graphics.Paint paint=new android.graphics.Paint(3);
            android.graphics.Path mask=new android.graphics.Path();mask.setFillType(android.graphics.Path.FillType.EVEN_ODD);mask.addRect(0,0,getWidth(),getHeight(),android.graphics.Path.Direction.CW);mask.addRoundRect(x,y,x+size,y+size,dp(20),dp(20),android.graphics.Path.Direction.CW);paint.setColor(0x99081020);canvas.drawPath(mask,paint);
            paint.setColor(0xffffffff);paint.setStyle(android.graphics.Paint.Style.STROKE);paint.setStrokeWidth(dp(3));paint.setStrokeCap(android.graphics.Paint.Cap.ROUND);float arm=dp(30),radius=dp(18);android.graphics.Path corners=new android.graphics.Path();corners.moveTo(x,y+arm);corners.lineTo(x,y+radius);corners.quadTo(x,y,x+radius,y);corners.lineTo(x+arm,y);corners.moveTo(x+size-arm,y);corners.lineTo(x+size-radius,y);corners.quadTo(x+size,y,x+size,y+radius);corners.lineTo(x+size,y+arm);corners.moveTo(x+size,y+size-arm);corners.lineTo(x+size,y+size-radius);corners.quadTo(x+size,y+size,x+size-radius,y+size);corners.lineTo(x+size-arm,y+size);corners.moveTo(x+arm,y+size);corners.lineTo(x+radius,y+size);corners.quadTo(x,y+size,x,y+size-radius);corners.lineTo(x,y+size-arm);canvas.drawPath(corners,paint);
        }
    }
    private void close(){if(camera!=null){camera.setPreviewCallbackWithBuffer(null);camera.stopPreview();camera.release();camera=null;}}
    @Override protected void onDestroy(){close();decoder.shutdown();super.onDestroy();}
}
