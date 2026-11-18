// SPDX-License-Identifier: MPL-2.0
package app.relay.companion;

import android.app.Activity;
import android.content.Context;
import android.content.res.ColorStateList;
import android.graphics.Canvas;
import android.graphics.Color;
import android.graphics.Paint;
import android.graphics.drawable.GradientDrawable;
import android.graphics.drawable.RippleDrawable;
import android.net.ConnectivityManager;
import android.net.NetworkCapabilities;
import android.view.Gravity;
import android.view.View;
import android.widget.Button;
import android.widget.CheckBox;
import android.widget.EditText;
import android.widget.ImageView;
import android.widget.LinearLayout;
import android.widget.TextView;

/** Nearby computers and direct QR pairing in a compact Relay-specific layout. */
public class DiscoveryScreen extends LinearLayout {
    public final LinearLayout devices;
    public final TextView status;
    public final EditText link;
    public final CheckBox audio;
    public final TextView deviceIdentity;
    public final TextView settingsButton;
    private final Radar radar;
    private final int blue=0xff3471f6,ink=0xff1e2b44,muted=0xff77839a;

    private int dp(int n){return (int)(getResources().getDisplayMetrics().density*n);}
    private TextView text(String s,int size,int color){TextView t=new TextView(getContext());t.setText(s);t.setTextSize(size);t.setTextColor(color);return t;}
    private GradientDrawable shape(int color,int radius){GradientDrawable d=new GradientDrawable();d.setColor(color);d.setCornerRadius(dp(radius));return d;}
    private Button button(String title,boolean primary){Button b=new Button(getContext());b.setText(title);b.setAllCaps(false);b.setTextSize(13);b.setMinHeight(dp(48));b.setTextColor(primary?blue:muted);GradientDrawable bg=shape(primary?0xffedf3ff:Color.WHITE,11);if(!primary)bg.setStroke(dp(1),0xffdfe7f4);b.setBackground(new RippleDrawable(ColorStateList.valueOf(0x223471f6),bg,null));return b;}
    private void gap(LinearLayout parent,int height){parent.addView(new View(getContext()),new LayoutParams(1,dp(height)));}

    public DiscoveryScreen(Activity activity,Runnable scan,Runnable refresh){
        super(activity);setOrientation(VERTICAL);setBackgroundColor(0xfff5f8ff);

        LinearLayout header=new LinearLayout(activity);header.setOrientation(VERTICAL);header.setPadding(dp(20),dp(15),dp(20),dp(15));header.setBackgroundColor(Color.WHITE);
        LinearLayout brandRow=new LinearLayout(activity);brandRow.setGravity(Gravity.CENTER_VERTICAL);
        ImageView logo=new ImageView(activity);logo.setImageResource(getResources().getIdentifier("ic_relay_vector","drawable",activity.getPackageName()));logo.setContentDescription("Relay");LayoutParams logoLayout=new LayoutParams(dp(36),dp(36));logoLayout.rightMargin=dp(10);brandRow.addView(logo,logoLayout);
        LinearLayout brandCopy=new LinearLayout(activity);brandCopy.setOrientation(VERTICAL);TextView brand=text("Relay",21,ink);brand.setTypeface(null,1);brandCopy.addView(brand);brandCopy.addView(text("Screen sharing",12,muted));brandRow.addView(brandCopy,new LayoutParams(0,-2,1));
        settingsButton=text("Settings",13,blue);settingsButton.setGravity(Gravity.CENTER);settingsButton.setPadding(dp(12),dp(9),dp(12),dp(9));settingsButton.setBackground(shape(0xfff2f6ff,10));settingsButton.setContentDescription("Open Relay settings");brandRow.addView(settingsButton);header.addView(brandRow);
        deviceIdentity=text("This phone · rename",12,muted);deviceIdentity.setPadding(dp(46),dp(7),0,0);header.addView(deviceIdentity);addView(header);

        LinearLayout content=new LinearLayout(activity);content.setOrientation(VERTICAL);content.setPadding(dp(18),dp(21),dp(18),dp(24));addView(content,new LayoutParams(-1,-2));
        LinearLayout titleRow=new LinearLayout(activity);titleRow.setGravity(Gravity.CENTER_VERTICAL);TextView nearbyTitle=text("Nearby devices",21,ink);nearbyTitle.setTypeface(null,1);titleRow.addView(nearbyTitle,new LayoutParams(0,-2,1));
        Button search=button("Refresh",false);search.setOnClickListener(v->refresh.run());titleRow.addView(search,new LayoutParams(-2,-2));content.addView(titleRow);

        ConnectivityManager manager=(ConnectivityManager)activity.getSystemService(Context.CONNECTIVITY_SERVICE);NetworkCapabilities capabilities=manager==null?null:manager.getNetworkCapabilities(manager.getActiveNetwork());
        TextView network=text(capabilities==null||!capabilities.hasTransport(NetworkCapabilities.TRANSPORT_WIFI)?"Connect both devices to the same Wi-Fi":"Searching this Wi-Fi network",12,muted);network.setPadding(dp(2),dp(5),0,dp(13));content.addView(network);

        LinearLayout searching=new LinearLayout(activity);searching.setGravity(Gravity.CENTER_VERTICAL);searching.setPadding(dp(1),0,0,dp(10));radar=new Radar(activity);searching.addView(radar,new LayoutParams(dp(30),dp(30)));
        status=text("Looking for Relay computers…",13,ink);status.setGravity(Gravity.CENTER_VERTICAL);status.setPadding(dp(9),0,0,0);searching.addView(status);content.addView(searching);
        devices=new LinearLayout(activity);devices.setOrientation(VERTICAL);content.addView(devices);

        LinearLayout qrPanel=new LinearLayout(activity);qrPanel.setGravity(Gravity.CENTER_VERTICAL);qrPanel.setPadding(dp(15),dp(13),dp(12),dp(13));qrPanel.setBackground(shape(Color.WHITE,14));LayoutParams qrLayout=new LayoutParams(-1,-2);qrLayout.topMargin=dp(17);content.addView(qrPanel,qrLayout);
        LinearLayout qrCopy=new LinearLayout(activity);qrCopy.setOrientation(VERTICAL);TextView qrTitle=text("Pair with QR",15,ink);qrTitle.setTypeface(null,1);qrCopy.addView(qrTitle);TextView qrInfo=text("Scan the code shown on your computer.",12,muted);qrInfo.setPadding(0,dp(4),0,0);qrCopy.addView(qrInfo);qrPanel.addView(qrCopy,new LayoutParams(0,-2,1));
        Button camera=button("Scan",true);camera.setOnClickListener(v->scan.run());qrPanel.addView(camera,new LayoutParams(-2,-2));
        link=new EditText(activity);link.setSaveEnabled(false);link.setVisibility(GONE);

        LinearLayout soundPanel=new LinearLayout(activity);soundPanel.setGravity(Gravity.CENTER_VERTICAL);soundPanel.setPadding(dp(13),dp(5),dp(13),dp(5));soundPanel.setBackground(shape(Color.WHITE,12));audio=new CheckBox(activity);audio.setText("Share device audio");audio.setTextSize(14);audio.setTextColor(ink);audio.setButtonTintList(ColorStateList.valueOf(blue));audio.setMinHeight(dp(48));soundPanel.addView(audio);LayoutParams soundLayout=new LayoutParams(-1,-2);soundLayout.topMargin=dp(12);content.addView(soundPanel,soundLayout);
        TextView footer=text("Your screen is shared only after you approve it.",12,muted);footer.setGravity(Gravity.CENTER);footer.setPadding(dp(10),dp(16),dp(10),0);content.addView(footer);
    }

    public void scanning(boolean value){radar.scanning=value;radar.invalidate();}

    public View computer(String name,boolean remembered,Runnable select){
        LinearLayout row=new LinearLayout(getContext());row.setGravity(Gravity.CENTER_VERTICAL);row.setPadding(dp(12),dp(12),dp(12),dp(12));row.setBackground(shape(remembered?0xffedf3ff:Color.WHITE,13));row.setForeground(new RippleDrawable(ColorStateList.valueOf(0x173471f6),null,shape(Color.WHITE,13)));row.setClickable(true);row.setFocusable(true);
        row.addView(new Monitor(getContext(),remembered),new LayoutParams(dp(38),dp(38)));LinearLayout copy=new LinearLayout(getContext());copy.setOrientation(VERTICAL);copy.setPadding(dp(11),0,dp(6),0);TextView title=text(name,15,ink);title.setTypeface(null,1);copy.addView(title);copy.addView(text(remembered?"Paired · tap to connect":"Nearby · pair first",10,remembered?blue:muted));row.addView(copy,new LayoutParams(0,-2,1));TextView arrow=text("›",23,blue);arrow.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);row.addView(arrow);row.setOnClickListener(v->select.run());row.setContentDescription("Connect to "+name+(remembered?", paired device":", needs pairing"));
        LayoutParams layout=new LayoutParams(-1,-2);layout.bottomMargin=dp(8);row.setLayoutParams(layout);return row;
    }

    private class Monitor extends View {
        private final boolean trusted;
        Monitor(Context c,boolean trusted){super(c);this.trusted=trusted;setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);}
        @Override protected void onDraw(Canvas c){float k=getWidth()/38f;c.save();c.scale(k,k);Paint p=new Paint(3);p.setColor(trusted?blue:0xff8b9bb5);c.drawCircle(19,19,19,p);p.setColor(Color.WHITE);c.drawRoundRect(9,11,29,25,3,3,p);c.drawLine(19,25,19,29,p);c.drawLine(14,29,24,29,p);c.restore();}
    }

    private class Radar extends View {
        boolean scanning=true;private final long start=android.os.SystemClock.uptimeMillis();
        Radar(Context c){super(c);setContentDescription("Searching for nearby Relay computers");}
        @Override protected void onDraw(Canvas c){
            Paint p=new Paint(3);float cx=getWidth()/2f,cy=getHeight()/2f;p.setStyle(Paint.Style.STROKE);p.setStrokeWidth(dp(1));
            if(scanning){float phase=((android.os.SystemClock.uptimeMillis()-start)%1700)/1700f;for(int i=0;i<2;i++){float t=(phase+i/2f)%1f;p.setColor(Color.argb((int)(65*(1-t)),52,113,246));c.drawCircle(cx,cy,dp(4)+dp(10)*t,p);}}
            else{p.setColor(0x553471f6);c.drawCircle(cx,cy,dp(9),p);}
            p.setStyle(Paint.Style.FILL);p.setColor(blue);c.drawCircle(cx,cy,dp(3),p);if(scanning&&isShown())postInvalidateDelayed(50);
        }
    }
}
