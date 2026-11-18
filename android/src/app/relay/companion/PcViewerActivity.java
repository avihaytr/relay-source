// SPDX-License-Identifier: MPL-2.0
package app.relay.companion;
import android.app.Activity;
import android.os.*;
import android.content.*;
import android.graphics.Color;
import android.media.*;
import android.net.Uri;
import android.view.*;
import android.widget.*;
import java.io.*;
import java.nio.ByteBuffer;
import javax.net.ssl.SSLSocket;

/** View-only client: no desktop keyboard, mouse or touch commands are sent. */
public class PcViewerActivity extends Activity implements SurfaceHolder.Callback {
    private FrameLayout root;
    private SurfaceView video;
    private LinearLayout controls;
    private TextView waiting;
    private volatile boolean stopped,started,seenFrame;
    private volatile SSLSocket socket;
    private int videoWidth=1280,videoHeight=720;
    private long frames;
    private final Handler main=new Handler(Looper.getMainLooper());
    private final Runnable hideControls=()->{if(seenFrame)controls.setVisibility(View.GONE);};
    private int dp(int x){return (int)(getResources().getDisplayMetrics().density*x);}
    @Override public void onCreate(Bundle state){
        super.onCreate(state);getWindow().addFlags(WindowManager.LayoutParams.FLAG_KEEP_SCREEN_ON);getWindow().getDecorView().setSystemUiVisibility(View.SYSTEM_UI_FLAG_FULLSCREEN|View.SYSTEM_UI_FLAG_HIDE_NAVIGATION|View.SYSTEM_UI_FLAG_IMMERSIVE_STICKY);
        root=new FrameLayout(this);root.setBackgroundColor(0xff0f131b);video=new SurfaceView(this);root.addView(video,new FrameLayout.LayoutParams(-1,-1,Gravity.CENTER));video.getHolder().addCallback(this);
        waiting=new TextView(this);waiting.setText("Approve in Relay on your PC if requested.\nThen choose a screen or window in the sharing prompt.");waiting.setTextColor(Color.WHITE);waiting.setTextSize(17);waiting.setGravity(Gravity.CENTER);waiting.setPadding(dp(25),dp(20),dp(25),dp(20));root.addView(waiting,new FrameLayout.LayoutParams(-1,-1));
        controls=new LinearLayout(this);controls.setGravity(Gravity.CENTER_VERTICAL);controls.setPadding(dp(18),dp(9),dp(15),dp(9));controls.setBackgroundColor(0xd9111928);
        TextView title=new TextView(this);title.setText(getIntent().getStringExtra("receiverName")+"  ·  View only");title.setTextColor(Color.WHITE);title.setTextSize(15);controls.addView(title,new LinearLayout.LayoutParams(0,-2,1));
        Button stop=new Button(this);stop.setText("Disconnect");stop.setAllCaps(false);stop.setOnClickListener(v->finish());controls.addView(stop,new LinearLayout.LayoutParams(-2,dp(45)));root.addView(controls,new FrameLayout.LayoutParams(-1,-2,Gravity.TOP));setContentView(root);
        video.setOnTouchListener((view,event)->{if(event.getAction()==android.view.MotionEvent.ACTION_UP){controls.setVisibility(controls.getVisibility()==View.VISIBLE?View.GONE:View.VISIBLE);main.removeCallbacks(hideControls);main.postDelayed(hideControls,3500);}return true;});
        root.addOnLayoutChangeListener((view,l,t,r,b,ol,ot,or,ob)->fitVideo());
    }
    private void fitVideo(){int width=root.getWidth(),height=root.getHeight();if(width==0||height==0)return;float scale=Math.min((float)width/videoWidth,(float)height/videoHeight);FrameLayout.LayoutParams layout=(FrameLayout.LayoutParams)video.getLayoutParams();int w=(int)(videoWidth*scale),h=(int)(videoHeight*scale);if(layout.width!=w||layout.height!=h){layout.width=w;layout.height=h;layout.gravity=Gravity.CENTER;video.setLayoutParams(layout);}}
    @Override public void surfaceCreated(SurfaceHolder holder){if(!started){started=true;new Thread(()->receive(holder.getSurface()),"relay-pc-viewer").start();}}
    @Override public void surfaceChanged(SurfaceHolder holder,int format,int width,int height){}
    @Override public void surfaceDestroyed(SurfaceHolder holder){stopViewer();}
    private void receive(Surface surface){
        MediaCodec decoder=null;
        try{
            RelayConnection.configure(this);Uri target=Uri.parse(getIntent().getStringExtra("link"));int quality=Math.max(0,Math.min(2,Integer.parseInt(target.getQueryParameter("q")==null?"2":target.getQueryParameter("q"))));String phoneName=getIntent().getStringExtra("deviceName");if(phoneName==null||phoneName.isEmpty())phoneName=android.os.Build.MODEL;
            socket=RelayConnection.open(target,"view",phoneName,pendingSocket->{socket=pendingSocket;if(stopped)try{pendingSocket.close();}catch(Exception ignored){}});if(stopped)return;
            DataInputStream input=new DataInputStream(socket.getInputStream());byte[] magic=new byte[4];input.readFully(magic);String protocol=new String(magic,"UTF-8");
            if("ERR!".equals(protocol)){int size=input.readInt();if(size<0||size>4096)throw new IOException("Invalid sharing response");byte[] message=new byte[size];input.readFully(message);throw new IOException(new String(message,"UTF-8"));}
            if(!"PCV1".equals(protocol))throw new IOException("Update Relay on your computer to use PC-to-phone sharing");
            int width=input.readInt(),height=input.readInt(),fps=input.readInt();if(width<1||height<1||width>4096||height>4096||fps<1||fps>120)throw new IOException("Unsupported desktop video format");
            videoWidth=width;videoHeight=height;main.post(this::fitVideo);socket.setSoTimeout(12000);
            MediaFormat format=MediaFormat.createVideoFormat("video/avc",width,height);format.setInteger(MediaFormat.KEY_MAX_INPUT_SIZE,4*1024*1024);if(Build.VERSION.SDK_INT>=30)format.setInteger(MediaFormat.KEY_LOW_LATENCY,1);
            decoder=MediaCodec.createDecoderByType("video/avc");decoder.configure(format,surface,null,0);decoder.start();android.util.Log.i("Relay","PC viewer decoder: "+decoder.getName());
            MediaCodec.BufferInfo info=new MediaCodec.BufferInfo();byte[] packet=new byte[256*1024];long sequence=0;
            while(!stopped){
                int size=input.readInt();if(size==0)break;if(size<0||size>4*1024*1024)throw new IOException("Invalid desktop frame");if(packet.length<size)packet=new byte[size];input.readFully(packet,0,size);
                int index;
                while((index=decoder.dequeueInputBuffer(10000))<0&&!stopped)drain(decoder,info);
                if(stopped)break;ByteBuffer buffer=decoder.getInputBuffer(index);if(buffer==null||buffer.capacity()<size)throw new IOException("Desktop frame exceeds decoder input capacity");buffer.clear();buffer.put(packet,0,size);decoder.queueInputBuffer(index,0,size,sequence++*1000000/fps,0);drain(decoder,info);
            }
        }catch(Exception error){if(!stopped){android.util.Log.w("Relay","PC viewing ended",error);if(!seenFrame)runOnUiThread(()->setResult(RESULT_CANCELED,new Intent().putExtra("error",error.getMessage()==null?"PC sharing was not started. Choose a screen on your computer.":error.getMessage())));}}
        finally{stopViewer();if(decoder!=null){try{decoder.stop();}catch(Exception ignored){}decoder.release();}android.util.Log.i("Relay","PC viewer ended after "+frames+" frames");runOnUiThread(this::finish);}
    }
    private void drain(MediaCodec decoder,MediaCodec.BufferInfo info){
        for(;;){int index=decoder.dequeueOutputBuffer(info,0);if(index>=0){decoder.releaseOutputBuffer(index,true);frames++;if(!seenFrame){seenFrame=true;android.util.Log.i("Relay","First PC frame displayed");main.post(()->{waiting.setVisibility(View.GONE);main.postDelayed(hideControls,3500);});}}
            else if(index==MediaCodec.INFO_OUTPUT_FORMAT_CHANGED){MediaFormat format=decoder.getOutputFormat();videoWidth=format.getInteger(MediaFormat.KEY_WIDTH);videoHeight=format.getInteger(MediaFormat.KEY_HEIGHT);if(format.containsKey("crop-right")&&format.containsKey("crop-left"))videoWidth=format.getInteger("crop-right")-format.getInteger("crop-left")+1;if(format.containsKey("crop-bottom")&&format.containsKey("crop-top"))videoHeight=format.getInteger("crop-bottom")-format.getInteger("crop-top")+1;main.post(this::fitVideo);}
            else break;
        }
    }
    private void stopViewer(){stopped=true;SSLSocket connection=socket;if(connection!=null)try{connection.close();}catch(Exception ignored){}}
    @Override protected void onStop(){stopViewer();super.onStop();}
    @Override protected void onDestroy(){main.removeCallbacksAndMessages(null);stopViewer();super.onDestroy();}
}
