// SPDX-License-Identifier: MPL-2.0
package app.relay.companion;

import android.content.Context;
import android.graphics.Color;
import android.graphics.drawable.GradientDrawable;
import android.view.Gravity;
import android.view.View;
import android.widget.Button;
import android.widget.LinearLayout;
import android.widget.TextView;

/** Compact, route-first choices for the two Relay sharing directions. */
public class DirectionScreen extends LinearLayout {
    public final TextView name,status;
    private final TextView phoneLabel;
    private final int blue=0xff3471f6,ink=0xff1e2b44,muted=0xff77839a;

    private int dp(int x){return (int)(getResources().getDisplayMetrics().density*x);}
    private TextView text(String value,int size,int color){TextView view=new TextView(getContext());view.setText(value);view.setTextColor(color);view.setTextSize(size);return view;}

    public DirectionScreen(Context context,Runnable back,Runnable send,Runnable watch,java.util.function.Supplier<String> getDeviceName){
        super(context);setOrientation(VERTICAL);setPadding(dp(20),dp(18),dp(20),dp(24));setBackgroundColor(0xfff5f8ff);

        LinearLayout bar=new LinearLayout(context);bar.setGravity(Gravity.CENTER_VERTICAL);
        Button previous=new Button(context);previous.setText("‹");previous.setTextSize(28);previous.setAllCaps(false);previous.setBackgroundColor(Color.TRANSPARENT);previous.setTextColor(blue);previous.setContentDescription("Back to nearby devices");previous.setOnClickListener(v->back.run());
        bar.addView(previous,new LayoutParams(dp(48),dp(48)));
        LinearLayout heading=new LinearLayout(context);heading.setOrientation(VERTICAL);
        TextView title=text("Screen direction",18,ink);title.setTypeface(null,1);heading.addView(title);bar.addView(heading,new LayoutParams(0,-2,1));addView(bar);

        status=text("Choose what to share",14,muted);status.setPadding(dp(4),dp(10),0,dp(18));addView(status);

        LinearLayout route=new LinearLayout(context);route.setGravity(Gravity.CENTER_VERTICAL);route.setPadding(dp(14),dp(12),dp(14),dp(12));
        GradientDrawable routeBg=new GradientDrawable();routeBg.setColor(Color.WHITE);routeBg.setCornerRadius(dp(13));route.setBackground(routeBg);
        LinearLayout phone=new LinearLayout(context);phone.setOrientation(VERTICAL);
        TextView phoneKind=text("PHONE",10,muted);phoneKind.setTypeface(null,1);phone.addView(phoneKind);
        phoneLabel=text("This phone · "+getDeviceName.get(),13,ink);phoneLabel.setMaxLines(2);phone.addView(phoneLabel);
        route.addView(phone,new LayoutParams(0,-2,1));
        TextView bothWays=text("↔",23,blue);bothWays.setPadding(dp(8),0,dp(8),0);bothWays.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);route.addView(bothWays);
        LinearLayout computer=new LinearLayout(context);computer.setOrientation(VERTICAL);computer.setGravity(Gravity.RIGHT);
        TextView computerKind=text("COMPUTER",10,muted);computerKind.setTypeface(null,1);computer.addView(computerKind);
        name=text("Your computer",13,ink);name.setMaxLines(2);computer.addView(name);
        route.addView(computer,new LayoutParams(0,-2,1));addView(route);

        addView(new View(context),new LayoutParams(1,dp(20)));
        addChoice("01","Mirror phone to PC","Send your phone screen and supported audio","PHONE  →  PC",send);
        addChoice("02","Mirror PC to phone","View a screen or window selected on your computer","PC  →  PHONE",watch);
    }

    public void setDeviceLabel(String value){phoneLabel.setText("This phone · "+value);}

    private void addChoice(String index,String title,String subtitle,String route,Runnable action){
        LinearLayout row=new LinearLayout(getContext());row.setGravity(Gravity.CENTER_VERTICAL);row.setPadding(dp(14),dp(13),dp(14),dp(13));row.setClickable(true);row.setFocusable(true);
        GradientDrawable background=new GradientDrawable();background.setColor(Color.WHITE);background.setCornerRadius(dp(14));row.setBackground(background);row.setOnClickListener(v->action.run());
        row.setContentDescription(title+". "+subtitle+". "+route.replace("  "," "));
        TextView number=text(index,12,blue);number.setTypeface(null,1);number.setGravity(Gravity.CENTER);GradientDrawable mark=new GradientDrawable();mark.setColor(0xffedf3ff);mark.setCornerRadius(dp(9));number.setBackground(mark);row.addView(number,new LayoutParams(dp(38),dp(38)));
        LinearLayout copy=new LinearLayout(getContext());copy.setOrientation(VERTICAL);copy.setPadding(dp(13),0,dp(6),0);
        TextView heading=text(title,16,ink);heading.setTypeface(null,1);copy.addView(heading);
        TextView detail=text(subtitle,12,muted);detail.setPadding(0,dp(4),0,0);copy.addView(detail);
        TextView direction=text(route,10,blue);direction.setTypeface(null,1);direction.setPadding(0,dp(8),0,0);copy.addView(direction);
        row.addView(copy,new LayoutParams(0,-2,1));TextView arrow=text("›",25,blue);row.addView(arrow);LayoutParams layout=new LayoutParams(-1,-2);layout.bottomMargin=dp(10);addView(row,layout);
    }
}
