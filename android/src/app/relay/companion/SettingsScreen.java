// SPDX-License-Identifier: MPL-2.0
package app.relay.companion;

import android.content.Context;
import android.graphics.Color;
import android.graphics.drawable.GradientDrawable;
import android.view.Gravity;
import android.view.View;
import android.widget.*;

/** Persistent phone identity and preferred Wi-Fi stream quality. */
public final class SettingsScreen extends LinearLayout {
    public final EditText deviceName;
    private final RadioGroup quality;
    public final CheckBox requireApproval;
    private final LinearLayout trustedList;
    private int dp(int n){return (int)(getResources().getDisplayMetrics().density*n);}
    private TextView text(String s,int size,int color){TextView t=new TextView(getContext());t.setText(s);t.setTextSize(size);t.setTextColor(color);return t;}
    public SettingsScreen(Context c,Runnable back,Runnable save){
        super(c);setOrientation(VERTICAL);setFocusableInTouchMode(true);setPadding(dp(20),dp(22),dp(20),dp(24));setBackgroundColor(0xfff5f8ff);
        LinearLayout bar=new LinearLayout(c);bar.setGravity(Gravity.CENTER_VERTICAL);bar.setPadding(dp(3),dp(6),dp(10),dp(6));bar.setBackgroundColor(Color.TRANSPARENT);Button previous=new Button(c);previous.setText("‹");previous.setTextSize(28);previous.setTextColor(0xff3471f6);previous.setBackgroundColor(Color.TRANSPARENT);previous.setOnClickListener(v->back.run());bar.addView(previous,new LayoutParams(dp(44),dp(48)));TextView title=text("Settings",24,0xff1e2b44);title.setTypeface(null,1);bar.addView(title);addView(bar);
        TextView nameTitle=text("Device name",16,0xff1e2b44);nameTitle.setTypeface(null,1);nameTitle.setPadding(dp(4),dp(24),0,dp(6));addView(nameTitle);deviceName=new EditText(c);deviceName.setSingleLine();deviceName.setTextSize(16);deviceName.setHint("For example, Funky Gecko");deviceName.setInputType(1|0x2000);deviceName.setTextColor(0xff1e2b44);deviceName.setBackgroundTintList(android.content.res.ColorStateList.valueOf(0xff3471f6));addView(deviceName,new LayoutParams(-1,dp(52)));
        TextView nameHint=text("Helps you recognize this phone on your computer.",13,0xff77839a);nameHint.setPadding(dp(4),dp(4),0,dp(18));addView(nameHint);
        TextView qualityTitle=text("Wi-Fi stream quality",16,0xff1e2b44);qualityTitle.setTypeface(null,1);qualityTitle.setPadding(dp(4),dp(8),0,dp(6));addView(qualityTitle);quality=new RadioGroup(c);String[] titles={"Low latency", "HD · 60 FPS", "Full HD · 60 FPS"};String[] descriptions={"720p · 30 FPS · 4 Mbps", "720p · 8 Mbps", "Up to 1080p · 16 Mbps"};for(int i=0;i<3;i++){RadioButton option=new RadioButton(c);option.setId(700+i);option.setText(titles[i]+"\n"+descriptions[i]);option.setTextSize(15);option.setTextColor(0xff1e2b44);option.setButtonTintList(android.content.res.ColorStateList.valueOf(0xff3471f6));option.setPadding(dp(12),dp(12),dp(12),dp(12));option.setMinHeight(dp(76));android.graphics.drawable.GradientDrawable surface=new android.graphics.drawable.GradientDrawable();surface.setColor(Color.WHITE);surface.setCornerRadius(dp(12));option.setBackground(surface);RadioGroup.LayoutParams row=new RadioGroup.LayoutParams(-1,-2);row.bottomMargin=dp(8);quality.addView(option,row);}addView(quality,new LayoutParams(-1,-2));TextView detail=text("The phone and computer use the lower of their selected quality profiles. Higher profiles need more Wi-Fi bandwidth and battery.",13,0xff77839a);detail.setPadding(dp(4),dp(6),dp(4),dp(18));addView(detail);
        TextView trustedTitle=text("Trusted Devices",18,0xff1e2b44);trustedTitle.setTypeface(null,1);trustedTitle.setPadding(dp(4),dp(12),0,dp(8));addView(trustedTitle);
        requireApproval=new CheckBox(c);requireApproval.setText("Require approval for every session");requireApproval.setTextColor(0xff1e2b44);addView(requireApproval);
        TextView trustHint=text("New devices always need QR verification. Forget revokes saved trust and disconnects that computer.",13,0xff77839a);trustHint.setPadding(dp(4),dp(4),dp(4),dp(12));addView(trustHint);
        trustedList=new LinearLayout(c);trustedList.setOrientation(VERTICAL);addView(trustedList);
        TextView reset=text("Reset this phone’s identity…",13,0xff3471f6);reset.setPadding(dp(4),dp(12),dp(4),dp(18));reset.setOnClickListener(v->new android.app.AlertDialog.Builder(c).setTitle("Reset device identity?").setMessage("This retires this phone’s key and forgets every computer. You must pair again with QR and approval on each PC.").setNegativeButton("Cancel",null).setPositiveButton("Reset identity",(dialog,which)->{try{DeviceTrust.resetIdentity();refreshTrust();}catch(Exception error){Toast.makeText(c,"Could not reset device identity",Toast.LENGTH_LONG).show();}}).show());addView(reset);
        Button saveButton=new Button(c);saveButton.setText("Save settings");saveButton.setAllCaps(false);saveButton.setTextColor(Color.WHITE);GradientDrawable bg=new GradientDrawable();bg.setColor(0xff3471f6);bg.setCornerRadius(dp(12));saveButton.setBackground(bg);saveButton.setOnClickListener(v->save.run());addView(saveButton,new LayoutParams(-1,dp(52)));
    }
    public void setValues(String name,int profile){deviceName.setText(name);deviceName.setSelection(deviceName.length());quality.check(700+Math.max(0,Math.min(2,profile)));}
    public void refreshTrust(){trustedList.removeAllViews();try{requireApproval.setChecked(DeviceTrust.requireApproval());org.json.JSONObject peers=DeviceTrust.peers();if(peers.length()==0)trustedList.addView(text("No remembered computers yet.",14,0xff77839a));java.util.Iterator<String> ids=peers.keys();while(ids.hasNext()){String id=ids.next();LinearLayout row=new LinearLayout(getContext());row.setGravity(Gravity.CENTER_VERTICAL);LinearLayout copy=new LinearLayout(getContext());copy.setOrientation(VERTICAL);copy.addView(text(peers.getJSONObject(id).getString("name"),15,0xff1e2b44));copy.addView(text(id.substring(0,12)+"…",12,0xff77839a));row.addView(copy,new LayoutParams(0,-2,1));Button forget=new Button(getContext());forget.setText("Forget Device");forget.setAllCaps(false);forget.setTextSize(12);forget.setOnClickListener(v->new android.app.AlertDialog.Builder(getContext()).setTitle("Forget this computer?").setMessage("Future connections will require QR pairing again.").setNegativeButton("Cancel",null).setPositiveButton("Forget",(dialog,which)->{try{DeviceTrust.forget(id);refreshTrust();}catch(Exception error){Toast.makeText(getContext(),"Could not revoke trust",Toast.LENGTH_LONG).show();}}).show());row.addView(forget);trustedList.addView(row);}}catch(Exception error){trustedList.addView(text("Saved trust unavailable. Reset the identity to pair again.",14,0xffb43e47));}}
    public int profile(){return Math.max(0,quality.getCheckedRadioButtonId()-700);}
}
