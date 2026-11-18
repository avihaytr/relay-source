// SPDX-License-Identifier: MPL-2.0
package app.relay.companion;

import android.app.Activity;
import android.content.Context;
import android.content.res.ColorStateList;
import android.graphics.Canvas;
import android.graphics.Color;
import android.graphics.Paint;
import android.graphics.Path;
import android.graphics.RectF;
import android.graphics.drawable.GradientDrawable;
import android.graphics.drawable.RippleDrawable;
import android.view.Gravity;
import android.view.View;
import android.widget.Button;
import android.widget.LinearLayout;
import android.widget.TextView;

/** Relay's live-session dashboard; connection and audio behavior stay in ShareService. */
public class SessionScreen extends LinearLayout {
    private final int blue=0xff3471f6,ink=0xff1e2b44,mutedColor=0xff77839a;
    private final TextView state,name,audio,subtitle;
    private final Control mute;
    private int dp(int n){return (int)(getResources().getDisplayMetrics().density*n);}
    private TextView text(String value,int size,int color){TextView view=new TextView(getContext());view.setText(value);view.setTextColor(color);view.setTextSize(size);return view;}

    public SessionScreen(Activity activity,Runnable disconnect){
        super(activity);setOrientation(VERTICAL);setPadding(dp(22),dp(20),dp(22),dp(28));setBackgroundColor(0xfff5f8ff);

        LinearLayout bar=new LinearLayout(activity);bar.setGravity(Gravity.CENTER_VERTICAL);
        TextView brand=text("Relay",24,ink);brand.setTypeface(null,1);bar.addView(brand,new LayoutParams(0,-2,1));
        LinearLayout live=new LinearLayout(activity);live.setGravity(Gravity.CENTER_VERTICAL);live.setPadding(dp(11),dp(7),dp(11),dp(7));GradientDrawable liveBg=new GradientDrawable();liveBg.setColor(0xffe8f7f1);liveBg.setCornerRadius(dp(20));live.setBackground(liveBg);
        View dot=new View(activity);GradientDrawable dotBg=new GradientDrawable();dotBg.setColor(0xff27a878);dotBg.setShape(GradientDrawable.OVAL);dot.setBackground(dotBg);live.addView(dot,new LayoutParams(dp(7),dp(7)));
        state=text("Connected",12,0xff217c5c);state.setPadding(dp(7),0,0,0);live.addView(state);bar.addView(live);addView(bar);

        TextView eyebrow=text("SHARING WITH",10,mutedColor);eyebrow.setTypeface(null,1);eyebrow.setLetterSpacing(0.08f);eyebrow.setPadding(dp(2),dp(32),0,dp(7));addView(eyebrow);
        name=text("Your computer",24,ink);name.setTypeface(null,1);addView(name);
        subtitle=text("Your phone screen is live on this computer.",14,mutedColor);subtitle.setPadding(dp(2),dp(7),0,dp(22));addView(subtitle);

        LinearLayout audioPanel=new LinearLayout(activity);audioPanel.setGravity(Gravity.CENTER_VERTICAL);audioPanel.setPadding(dp(15),dp(14),dp(12),dp(14));GradientDrawable panelBg=new GradientDrawable();panelBg.setColor(Color.WHITE);panelBg.setCornerRadius(dp(15));audioPanel.setBackground(panelBg);
        Icon speaker=new Icon(activity);audioPanel.addView(speaker,new LayoutParams(dp(30),dp(30)));
        LinearLayout audioCopy=new LinearLayout(activity);audioCopy.setOrientation(VERTICAL);audioCopy.setPadding(dp(12),0,dp(8),0);
        TextView audioTitle=text("Device audio",15,ink);audioTitle.setTypeface(null,1);audioCopy.addView(audioTitle);
        audio=text("Checking audio…",12,mutedColor);audio.setPadding(0,dp(4),0,0);audioCopy.addView(audio);audioPanel.addView(audioCopy,new LayoutParams(0,-2,1));
        mute=new Control(activity);audioPanel.addView(mute,new LayoutParams(-2,-2));mute.setOnClickListener(v->{ShareService.toggleMuted();refresh();});addView(audioPanel);

        TextView audioNote=text("Some apps don’t allow their audio to be shared.",12,mutedColor);audioNote.setPadding(dp(2),dp(12),dp(2),0);addView(audioNote);
        TextView test=text("Test audio",13,blue);test.setPadding(dp(2),dp(12),dp(2),dp(24));test.setMinHeight(dp(48));test.setContentDescription("Play test sound through shared device audio");test.setOnClickListener(v->{if(ShareService.sharing&&ShareService.audioAvailable)ShareService.playTestSound();});addView(test);

        View divider=new View(activity);divider.setBackgroundColor(0xffe2e9f3);addView(divider,new LayoutParams(-1,dp(1)));
        Button stop=new Button(activity);stop.setText("Disconnect");stop.setAllCaps(false);stop.setTextSize(15);stop.setMinHeight(dp(48));stop.setTextColor(0xffa3424b);stop.setContentDescription("Disconnect screen sharing");
        GradientDrawable stopBg=new GradientDrawable();stopBg.setColor(0xfffff1f2);stopBg.setStroke(dp(1),0xfff0d6da);stopBg.setCornerRadius(dp(12));stop.setBackground(new RippleDrawable(ColorStateList.valueOf(0x18a3424b),stopBg,null));
        stop.setOnClickListener(v->disconnect.run());LayoutParams stopLayout=new LayoutParams(-1,-2);stopLayout.topMargin=dp(20);addView(stop,stopLayout);
        TextView footer=text("Screen sharing ends when you disconnect or stop it from the Android notification.",12,mutedColor);footer.setGravity(Gravity.CENTER);footer.setPadding(dp(10),dp(16),dp(10),0);addView(footer);
    }

    public void refresh(){
        boolean active=ShareService.sharing;
        state.setText(active?"Connected":"Connecting");name.setText(ShareService.receiverName);
        subtitle.setText(active?"Your phone screen is live on this computer.":"Authenticating… Approve on your PC if requested.");
        mute.setEnabled(active&&ShareService.audioAvailable);mute.setAlpha(mute.isEnabled()?1f:0.45f);
        mute.label.setText(ShareService.muted?"Unmute":"Mute");mute.icon.muted=ShareService.muted;mute.icon.invalidate();audio.setText(ShareService.audioStatus);
    }

    private final class Control extends LinearLayout {
        final TextView label;final Icon icon;
        Control(Context context){
            super(context);setGravity(Gravity.CENTER);setOrientation(HORIZONTAL);setPadding(dp(10),dp(8),dp(10),dp(8));setMinimumHeight(dp(48));setClickable(true);setFocusable(true);
            GradientDrawable background=new GradientDrawable();background.setColor(0xffedf3ff);background.setCornerRadius(dp(10));setBackground(new RippleDrawable(ColorStateList.valueOf(0x223471f6),background,null));
            icon=new Icon(context);addView(icon,new LayoutParams(dp(19),dp(19)));
            label=text("Mute",13,blue);label.setTypeface(null,1);label.setPadding(dp(5),0,0,0);addView(label);
            setContentDescription("Mute or unmute shared device audio");
        }
    }

    private final class Icon extends View {
        boolean muted;
        Icon(Context context){super(context);setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);}
        @Override protected void onDraw(Canvas canvas){
            canvas.save();canvas.scale(getWidth()/36f,getHeight()/36f);Paint p=new Paint(3);p.setStyle(Paint.Style.STROKE);p.setStrokeCap(Paint.Cap.ROUND);p.setStrokeJoin(Paint.Join.ROUND);p.setStrokeWidth(2);p.setColor(blue);
            Path speaker=new Path();speaker.moveTo(4,14);speaker.lineTo(10,14);speaker.lineTo(19,6);speaker.lineTo(19,30);speaker.lineTo(10,22);speaker.lineTo(4,22);speaker.close();canvas.drawPath(speaker,p);
            if(muted){canvas.drawLine(25,11,34,25,p);canvas.drawLine(25,25,34,11,p);}else{canvas.drawArc(new RectF(17,9,31,27),-65,130,false,p);canvas.drawArc(new RectF(13,4,36,32),-50,100,false,p);}
            canvas.restore();
        }
    }
}
