// SPDX-License-Identifier: MPL-2.0
package app.relay.companion;
import android.app.*;
import android.Manifest;
import android.content.pm.PackageManager;
import android.os.*;
import android.content.*;
import android.media.projection.MediaProjectionManager;
import android.net.Uri;
import android.net.nsd.*;
import android.net.wifi.WifiManager;
import java.net.*;
import java.util.*;
import android.graphics.Color;
import android.view.*;
import android.widget.*;

public class MainActivity extends Activity {
    private EditText link;
    private DiscoveryScreen discovery;
    private DirectionScreen direction;
    private View discoveryPane,directionPane;
    private boolean choosingDirection;
    private int requestedDirection,discoveryGeneration;
    private NsdManager nsd;
    private NsdManager.DiscoveryListener listener;
    private WifiManager.MulticastLock multicast;
    private LinearLayout nearby;
    private TextView discoveryStatus;
    private final ArrayDeque<NsdServiceInfo> queue=new ArrayDeque<>();
    private final HashMap<String,View> computers=new HashMap<>();
    private final HashMap<String,Endpoint> endpoints=new HashMap<>();
    private final Handler reachabilityHandler=new Handler(Looper.getMainLooper());
    private boolean probingNearby;
    private boolean needsDiscoveryRefresh;
    private long lastDiscoveryRefresh;
    private static final class Endpoint {final String name,connection,host;final int port;int misses;Endpoint(String name,String connection,String host,int port){this.name=name;this.connection=connection;this.host=host;this.port=port;}}
    private final Runnable reachabilityCheck=new Runnable(){public void run(){checkNearby();}};
    private boolean resolving;
    private boolean scanning;
    private String pending;
    private String selectedComputer="Your computer";
    private int selectedQuality=2;
    private String deviceName;
    private int phoneQuality=2;
    private boolean showingSettings;
    private SettingsScreen settingsScreen;
    private View settingsPane;
    private CheckBox shareAudio;
    private View setupPane;
    private SessionScreen sessionScreen;
    private boolean showingSession;
    private final Handler sessionHandler=new Handler(Looper.getMainLooper());
    private final Runnable sessionRefresh=new Runnable(){public void run(){refreshSession();sessionHandler.postDelayed(this,300);}};
    private boolean checkingConnection;
    private final int blue=Color.rgb(52,113,246),ink=Color.rgb(30,43,68),muted=Color.rgb(119,131,154);
    private int dp(int value){return (int)(getResources().getDisplayMetrics().density*value);}
    private String randomDeviceName(){String[] adjectives={"Amber","Brisk","Calm","Clever","Cloudy","Cozy","Cosmic","Daring","Dazzling","Gentle","Golden","Grumpy","Happy","Jolly","Lucky","Merry","Misty","Nimble","Quiet","Quick","Silver","Sunny","Velvet","Witty","Zealous","Bouncy","Curious","Dreamy","Funky","Fuzzy","Kind","Lively","Sleepy","Tiny","Wise"};String[] nouns={"Otter","Badger","Heron","Panda","Robin","Marten","Falcon","Koala","Lynx","Puffin","Dolphin","Gecko","Marmot","Raven","Turtle","Fox","Wombat","Finch","Orca","Lemur","Moose","Owl","Seal","Yak","Ferret","Hedgehog","Kingfisher","Osprey","Quokka","Sparrow","Walrus","Weasel","Penguin"};java.util.Random random=new java.security.SecureRandom();return adjectives[random.nextInt(adjectives.length)]+" "+nouns[random.nextInt(nouns.length)];}
    private void setDeviceName(String value){deviceName=value;getPreferences(0).edit().putString("deviceName",value).apply();discovery.deviceIdentity.setText("This phone · "+value);if(direction!=null)direction.setDeviceLabel(value);}
    private void showSettings(){if(ShareService.sharing||ShareService.connecting)return;showingSettings=true;stopDiscovery();setupPane.setVisibility(View.GONE);settingsPane.setVisibility(View.VISIBLE);settingsScreen.setValues(deviceName,phoneQuality);settingsScreen.refreshTrust();}
    private void saveSettings(){String value=settingsScreen.deviceName.getText().toString().trim();if(value.isEmpty()||value.getBytes(java.nio.charset.StandardCharsets.UTF_8).length>50){Toast.makeText(this,"Enter a name under 50 bytes",Toast.LENGTH_SHORT).show();return;}try{DeviceTrust.setApproval(settingsScreen.requireApproval.isChecked());}catch(Exception error){Toast.makeText(this,"Could not save trust settings",Toast.LENGTH_LONG).show();return;}phoneQuality=settingsScreen.profile();setDeviceName(value);getPreferences(0).edit().putInt("wifiQuality",phoneQuality).apply();showingSettings=false;settingsPane.setVisibility(View.GONE);setupPane.setVisibility(View.VISIBLE);showDetection();Toast.makeText(this,"Relay settings saved",Toast.LENGTH_SHORT).show();}
    private TextView text(String value,int size,int color){TextView view=new TextView(this);view.setText(value);view.setTextSize(size);view.setTextColor(color);return view;}
    private Button button(String title,boolean primary){Button button=new Button(this);button.setText(title);button.setAllCaps(false);button.setTextSize(16);button.setTextColor(primary?Color.WHITE:blue);android.graphics.drawable.GradientDrawable background=new android.graphics.drawable.GradientDrawable();background.setColor(primary?blue:Color.WHITE);background.setCornerRadius(dp(12));if(!primary)background.setStroke(dp(1),Color.rgb(220,229,246));button.setBackground(background);LinearLayout.LayoutParams layout=new LinearLayout.LayoutParams(-1,dp(54));layout.topMargin=dp(12);button.setLayoutParams(layout);return button;}
    private void gap(LinearLayout box,int height){View space=new View(this);box.addView(space,new LinearLayout.LayoutParams(1,dp(height)));}
    @Override public void onCreate(Bundle state) {
        super.onCreate(state);getWindow().setStatusBarColor(blue);getWindow().setNavigationBarColor(Color.WHITE);getWindow().getDecorView().setSystemUiVisibility(View.SYSTEM_UI_FLAG_LIGHT_NAVIGATION_BAR|View.SYSTEM_UI_FLAG_LIGHT_STATUS_BAR);
        try{RelayConnection.configure(this);}catch(Exception error){Toast.makeText(this,"Device identity unavailable. Pairing is disabled.",Toast.LENGTH_LONG).show();}
        nsd=(NsdManager)getSystemService(NSD_SERVICE);deviceName=getPreferences(0).getString("deviceName",null);if(deviceName==null){deviceName=randomDeviceName();getPreferences(0).edit().putString("deviceName",deviceName).apply();}phoneQuality=Math.max(0,Math.min(2,getPreferences(0).getInt("wifiQuality",2)));
        discovery=new DiscoveryScreen(this,()->startActivityForResult(new Intent(this,ScanActivity.class),43),()->{stopDiscovery();discover();});
        link=discovery.link;link.setSaveEnabled(false);nearby=discovery.devices;discoveryStatus=discovery.status;shareAudio=discovery.audio;discovery.deviceIdentity.setText("This phone · "+deviceName);
        shareAudio.setChecked(Build.VERSION.SDK_INT>=29&&getPreferences(0).getBoolean("shareAudio",true));shareAudio.setEnabled(Build.VERSION.SDK_INT>=29);shareAudio.setOnCheckedChangeListener((button,checked)->getPreferences(0).edit().putBoolean("shareAudio",checked).apply());
        ScrollView list=new ScrollView(this);list.setFillViewport(true);list.addView(discovery);discoveryPane=list;
        direction=new DirectionScreen(this,()->showDetection(),()->connectDirection(false),()->connectDirection(true),()->deviceName);direction.setDeviceLabel(deviceName);ScrollView choices=new ScrollView(this);choices.setFillViewport(true);choices.addView(direction);choices.setVisibility(View.GONE);directionPane=choices;
        FrameLayout setup=new FrameLayout(this);setup.addView(list,new FrameLayout.LayoutParams(-1,-1));setup.addView(choices,new FrameLayout.LayoutParams(-1,-1));setupPane=setup;
        FrameLayout root=new FrameLayout(this);root.setBackgroundColor(0xfff5f8ff);if(Build.VERSION.SDK_INT>=30)root.setOnApplyWindowInsetsListener((view,insets)->{android.graphics.Insets bars=insets.getInsets(android.view.WindowInsets.Type.systemBars()|android.view.WindowInsets.Type.displayCutout());view.setPadding(bars.left,bars.top,bars.right,bars.bottom);return insets;});root.addView(setup,new FrameLayout.LayoutParams(-1,-1));settingsScreen=new SettingsScreen(this,()->{showingSettings=false;settingsPane.setVisibility(View.GONE);setupPane.setVisibility(View.VISIBLE);showDetection();},this::saveSettings);settingsScreen.setValues(deviceName,phoneQuality);settingsScreen.refreshTrust();ScrollView settingsScroll=new ScrollView(this);settingsScroll.setFillViewport(true);settingsScroll.addView(settingsScreen);settingsScroll.setVisibility(View.GONE);settingsPane=settingsScroll;root.addView(settingsScroll,new FrameLayout.LayoutParams(-1,-1));discovery.settingsButton.setOnClickListener(v->showSettings());
        sessionScreen=new SessionScreen(this,()->{stopService(new Intent(this,ShareService.class));ShareService.sharing=false;ShareService.connecting=false;refreshSession();});ScrollView sessionScroll=new ScrollView(this);sessionScroll.setFillViewport(true);sessionScroll.addView(sessionScreen);sessionScroll.setVisibility(View.GONE);root.addView(sessionScroll,new FrameLayout.LayoutParams(-1,-1));sessionScreen.setTag(sessionScroll);setContentView(root);if(Build.VERSION.SDK_INT>=30)getWindow().getInsetsController().setSystemBarsAppearance(android.view.WindowInsetsController.APPEARANCE_LIGHT_STATUS_BARS|android.view.WindowInsetsController.APPEARANCE_LIGHT_NAVIGATION_BARS,android.view.WindowInsetsController.APPEARANCE_LIGHT_STATUS_BARS|android.view.WindowInsetsController.APPEARANCE_LIGHT_NAVIGATION_BARS);receive(getIntent());
    }
    private void chooseComputer(String name,String connection){
        if(connection.startsWith("relay-discovery://")){selectedComputer=name;final String discoveryConnection=connection;new AlertDialog.Builder(this).setTitle("Pair this computer").setMessage("Pairing saves trust on both devices. It does not start screen sharing. Choose the QR or six-digit PIN shown in Relay on this PC.").setNegativeButton("Cancel",null).setNeutralButton("Enter PIN",(dialog,which)->requestPin(discoveryConnection)).setPositiveButton("Scan QR",(dialog,which)->startActivityForResult(new Intent(this,ScanActivity.class),43)).show();return;}
        try{if(!RelayConnection.validTarget(Uri.parse(connection.trim())))throw new Exception();}
        catch(Exception error){Toast.makeText(this,"Scan the current pairing QR from your PC screen",1).show();return;}
        selectedComputer=name;connection=Uri.parse(connection).buildUpon().appendQueryParameter("computer",name).build().toString();link.setText(connection);Uri selected=Uri.parse(connection);try{selectedQuality=Math.min(phoneQuality,Math.max(0,Math.min(2,Integer.parseInt(selected.getQueryParameter("q")==null?"2":selected.getQueryParameter("q")))));}catch(Exception ignored){selectedQuality=phoneQuality;}choosingDirection=true;stopDiscovery();direction.name.setText(name);direction.setDeviceLabel(deviceName);direction.status.setText("Choose which screen to mirror");direction.status.setTextColor(muted);discoveryStatus=direction.status;discoveryPane.setVisibility(View.GONE);directionPane.setVisibility(View.VISIBLE);
    }
    private void showDetection(){showingSettings=false;choosingDirection=false;settingsPane.setVisibility(View.GONE);setupPane.setVisibility(View.VISIBLE);directionPane.setVisibility(View.GONE);discoveryPane.setVisibility(View.VISIBLE);discoveryStatus=discovery.status;discover();}
    private void connectDirection(boolean watch){requestedDirection=watch?1:0;try{if(RelayConnection.validTrusted(Uri.parse(link.getText().toString()))&&DeviceTrust.requireApproval()){new AlertDialog.Builder(this).setTitle("Start this session?").setMessage("Connect to the remembered computer you selected? Screen capture still requires platform consent.").setNegativeButton("Cancel",null).setPositiveButton("Connect",(dialog,which)->startSharing()).show();return;}}catch(Exception error){Toast.makeText(this,"Saved trust unavailable. Pair again.",Toast.LENGTH_LONG).show();return;}startSharing();}
    private void confirmFirstPair(String scanned){new AlertDialog.Builder(this).setTitle("Pair with this computer?").setMessage("Only continue if you scanned the QR directly from this PC. Relay will save this device as paired after you approve the request on the PC. No screen or audio capture starts during pairing.").setNegativeButton("Cancel",null).setPositiveButton("Pair device",(dialog,which)->startPairing(Uri.parse(scanned))).show();}
    private void requestPin(String discoveryUri){Uri endpoint=Uri.parse(discoveryUri);EditText input=new EditText(this);input.setSingleLine(true);input.setInputType(android.text.InputType.TYPE_CLASS_NUMBER|android.text.InputType.TYPE_NUMBER_VARIATION_PASSWORD);input.setFilters(new android.text.InputFilter[]{new android.text.InputFilter.LengthFilter(6)});input.setHint("6-digit PIN");input.setTextSize(22);input.setGravity(Gravity.CENTER);new AlertDialog.Builder(this).setTitle("Enter PC pairing PIN").setMessage("Enter the current six-digit PIN displayed in Relay on this computer.").setView(input).setNegativeButton("Cancel",null).setPositiveButton("Continue",(dialog,which)->{String value=input.getText().toString();if(!value.matches("[0-9]{6}")){Toast.makeText(this,"Enter all six digits",Toast.LENGTH_SHORT).show();return;}Uri pin=Uri.parse("relay-pin://"+endpoint.getHost()+":"+endpoint.getPort()+"/"+value+"?v=6&q="+endpoint.getQueryParameter("q"));pin=pin.buildUpon().appendQueryParameter("computer",selectedComputer).build();startPairing(pin);}).show();}
    private void startPairing(Uri target){
        if(!RelayConnection.validQr(target)&&!RelayConnection.validPin(target)){discoveryStatus.setText("Scan the current PC QR or enter its six-digit PIN.");return;}
        String qrName=target.getQueryParameter("computer");if(qrName!=null&&!qrName.isEmpty())selectedComputer=qrName;
        if(target.getQueryParameter("computer")==null)target=target.buildUpon().appendQueryParameter("computer",selectedComputer).build();
        discoveryStatus.setTextColor(muted);discoveryStatus.setText("Waiting for approval on the PC…");final Uri pairingTarget=target;
        final String pairedName=selectedComputer;
        new Thread(()->{try{String verifiedPin=RelayConnection.pair(pairingTarget,deviceName,socket->{},code->confirmPinCode(code));runOnUiThread(()->{showDetection();addPairedComputer(pairingTarget,pairedName,verifiedPin);Toast.makeText(this,"Paired with "+pairedName,Toast.LENGTH_SHORT).show();});}
            catch(Exception error){runOnUiThread(()->{discoveryStatus.setTextColor(Color.rgb(195,63,76));discoveryStatus.setText("Pairing was not approved or verified. Try a fresh PC QR or PIN.");});}},"relay-pair-device").start();
    }
    private boolean confirmPinCode(String code){java.util.concurrent.CountDownLatch latch=new java.util.concurrent.CountDownLatch(1);java.util.concurrent.atomic.AtomicBoolean accepted=new java.util.concurrent.atomic.AtomicBoolean(false);runOnUiThread(()->new AlertDialog.Builder(this).setTitle("Verify this computer").setMessage("Compare this code with the verification code shown in Relay on the PC. Continue only if they match.\n\n"+code).setNegativeButton("Codes differ",(dialog,which)->{accepted.set(false);latch.countDown();}).setPositiveButton("Codes match",(dialog,which)->{accepted.set(true);latch.countDown();}).setOnCancelListener(dialog->latch.countDown()).show());try{if(!latch.await(60,java.util.concurrent.TimeUnit.SECONDS))return false;}catch(InterruptedException error){Thread.currentThread().interrupt();return false;}return accepted.get();}
    private void addPairedComputer(Uri target,String name,String verifiedPin){try{String host=target.getHost();int port=target.getPort();String q=target.getQueryParameter("q");String connection=Uri.parse("relay-trusted://"+host+":"+port+"/"+verifiedPin+"?v=6&q="+(q==null?"2":q)).buildUpon().appendQueryParameter("computer",name).build().toString();Endpoint endpoint=new Endpoint(name,connection,host,port);endpoints.put(name,endpoint);View row=discovery.computer(name,true,()->chooseComputer(name,connection));View old=computers.put(name,row);if(old!=null)nearby.removeView(old);nearby.addView(row,0);discoveryStatus.setText("Paired device ready. Tap to choose a mirroring direction.");discoveryStatus.setTextColor(blue);}catch(Exception ignored){discoveryStatus.setText("Paired. Refresh nearby devices to connect.");}}
    @Override public void onBackPressed(){if(showingSettings&&!ShareService.sharing&&!ShareService.connecting){showDetection();}else if(choosingDirection&&!ShareService.sharing&&!ShareService.connecting){showDetection();}else{super.onBackPressed();}}
    @Override protected void onStart(){super.onStart();if(!ShareService.sharing&&!ShareService.connecting&&!choosingDirection&&!showingSettings)discover();}
    @Override protected void onResume(){super.onResume();sessionHandler.removeCallbacks(sessionRefresh);sessionRefresh.run();}
    @Override protected void onPause(){sessionHandler.removeCallbacks(sessionRefresh);super.onPause();}
    private void refreshSession(){
        boolean active=ShareService.sharing||ShareService.connecting;
        setupPane.setVisibility(active?View.GONE:showingSettings?View.GONE:View.VISIBLE);settingsPane.setVisibility(active||!showingSettings?View.GONE:View.VISIBLE);((View)sessionScreen.getTag()).setVisibility(active?View.VISIBLE:View.GONE);
        if(active){sessionScreen.refresh();if(!showingSession)stopDiscovery();}
        else if(showingSession){showDetection();if(!ShareService.lastError.isEmpty()){discoveryStatus.setText(ShareService.lastError);discoveryStatus.setTextColor(Color.rgb(195,63,76));}}
        showingSession=active;
    }
    @Override protected void onStop(){super.onStop();stopDiscovery();}
    private void stopDiscovery(){scanning=false;discoveryGeneration++;resolving=false;reachabilityHandler.removeCallbacks(reachabilityCheck);probingNearby=false;if(discovery!=null)discovery.scanning(false);if(listener!=null){try{nsd.stopServiceDiscovery(listener);}catch(Exception ignored){}listener=null;}if(multicast!=null&&multicast.isHeld())multicast.release();queue.clear();}
    private void discover(){
        if(scanning||choosingDirection||showingSettings)return;scanning=true;final int generation=++discoveryGeneration;discovery.scanning(true);computers.clear();endpoints.clear();nearby.removeAllViews();discoveryStatus.setTextColor(muted);discoveryStatus.setText("Searching for computers…");reachabilityHandler.removeCallbacks(reachabilityCheck);reachabilityHandler.postDelayed(reachabilityCheck,3500);
        WifiManager wifi=(WifiManager)getApplicationContext().getSystemService(WIFI_SERVICE);if(wifi!=null){multicast=wifi.createMulticastLock("Relay discovery");multicast.setReferenceCounted(false);multicast.acquire();}
        listener=new NsdManager.DiscoveryListener(){
            public void onDiscoveryStarted(String type){android.util.Log.i("Relay","Nearby discovery started");}
            public void onServiceFound(NsdServiceInfo info){runOnUiThread(()->{if(scanning&&generation==discoveryGeneration){queue.add(info);resolveNext();}});}
            public void onServiceLost(NsdServiceInfo info){runOnUiThread(()->{if(generation!=discoveryGeneration)return;removeComputer(info.getServiceName());});}
            public void onDiscoveryStopped(String type){}
            public void onStartDiscoveryFailed(String type,int error){runOnUiThread(()->discoveryStatus.setText("Discovery unavailable. Use the QR fallback."));}
            public void onStopDiscoveryFailed(String type,int error){}
        };
        try{nsd.discoverServices("_relay._tcp.",NsdManager.PROTOCOL_DNS_SD,listener);}catch(Exception e){discoveryStatus.setText("Discovery unavailable. Try the QR code tab.");}
    }
    private void resolveNext(){
        if(!scanning||resolving||queue.isEmpty())return;resolving=true;final int generation=discoveryGeneration;
        nsd.resolveService(queue.remove(),new NsdManager.ResolveListener(){
            public void onResolveFailed(NsdServiceInfo service,int error){runOnUiThread(()->{if(generation!=discoveryGeneration)return;resolving=false;resolveNext();});}
            public void onServiceResolved(NsdServiceInfo service){runOnUiThread(()->{
                resolving=false;
                try{
                    if(!scanning||generation!=discoveryGeneration)return;
                    byte[] version=service.getAttributes().get("version");if(version==null||!"6".equals(new String(version,"UTF-8")))return;
                    InetAddress host=service.getHost();if(Build.VERSION.SDK_INT>=34){for(InetAddress address:service.getHostAddresses())if(address instanceof Inet4Address){host=address;break;}}
                    if(!(host instanceof Inet4Address))return;
                    byte[] quality=service.getAttributes().get("quality");String q=quality==null?"2":new String(quality,"UTF-8");if(!q.matches("[012]"))q="2";
                    byte[] identity=service.getAttributes().get("id");String id=identity==null?"":new String(identity,"UTF-8");boolean remembered=false;try{remembered=DeviceTrust.trusted(id);}catch(Exception ignored){}
                    final String connection=remembered?"relay-trusted://"+host.getHostAddress()+":"+service.getPort()+"/"+id+"?v=6&q="+q:"relay-discovery://"+host.getHostAddress()+":"+service.getPort()+"?q="+q;
                    String name=service.getServiceName();Endpoint endpoint=new Endpoint(name,connection,host.getHostAddress(),service.getPort());verifyEndpoint(endpoint,generation);
                }catch(Exception e){android.util.Log.w("Relay","Discovery resolve failed",e);}finally{if(generation==discoveryGeneration)resolveNext();}
            });}
        });
    }
    private boolean endpointReachable(Endpoint endpoint){try(Socket probe=new Socket()){probe.connect(new InetSocketAddress(endpoint.host,endpoint.port),350);return true;}catch(Exception ignored){return false;}}
    private void verifyEndpoint(Endpoint endpoint,int generation){new Thread(()->{boolean reachable=endpointReachable(endpoint);runOnUiThread(()->{if(!scanning||generation!=discoveryGeneration)return;if(reachable){endpoints.put(endpoint.name,endpoint);View button=computers.get(endpoint.name);if(button==null){button=discovery.computer(endpoint.name,endpoint.connection.startsWith("relay-trusted://"),()->chooseComputer(endpoint.name,endpoint.connection));computers.put(endpoint.name,button);nearby.addView(button);}else{button.setOnClickListener(v->chooseComputer(endpoint.name,endpoint.connection));}discoveryStatus.setText(computers.size()+" computer"+(computers.size()==1?"":"s")+" nearby. Tap to connect.");android.util.Log.i("Relay","Reachable Relay computer: "+endpoint.name+" at "+endpoint.host+":"+endpoint.port);}else if(!endpoints.containsKey(endpoint.name)){android.util.Log.i("Relay","Ignored cached discovery record for unavailable computer "+endpoint.name+":"+endpoint.port);needsDiscoveryRefresh=true;refreshEmptyDiscovery(generation);}});},"relay-nearby-check").start();}
    private void refreshEmptyDiscovery(int generation){if(!needsDiscoveryRefresh||!scanning||generation!=discoveryGeneration||!computers.isEmpty()||!endpoints.isEmpty())return;long now=SystemClock.elapsedRealtime();if(lastDiscoveryRefresh!=0&&now-lastDiscoveryRefresh<1500)return;needsDiscoveryRefresh=false;lastDiscoveryRefresh=now;android.util.Log.i("Relay","Refreshing DNS-SD scan after an unavailable cached address");discoveryStatus.setText("Refreshing nearby computers…");stopDiscovery();reachabilityHandler.postDelayed(()->{if(!choosingDirection&&!showingSettings)discover();},250);}
    private void removeComputer(String name){endpoints.remove(name);View button=computers.remove(name);if(button!=null)nearby.removeView(button);if(computers.isEmpty()){discoveryStatus.setText("No computers found yet. Keep desktop Relay open.");needsDiscoveryRefresh=true;if(scanning)refreshEmptyDiscovery(discoveryGeneration);}else discoveryStatus.setText(computers.size()+" computer"+(computers.size()==1?"":"s")+" nearby. Tap to connect.");}
    private void checkNearby(){if(!scanning)return;if(probingNearby){reachabilityHandler.postDelayed(reachabilityCheck,2500);return;}if(endpoints.isEmpty()){refreshEmptyDiscovery(discoveryGeneration);if(scanning)reachabilityHandler.postDelayed(reachabilityCheck,3500);return;}probingNearby=true;int generation=discoveryGeneration;ArrayList<Endpoint> snapshot=new ArrayList<>(endpoints.values());new Thread(()->{HashMap<String,Boolean> results=new HashMap<>();for(Endpoint endpoint:snapshot)results.put(endpoint.name,endpointReachable(endpoint));runOnUiThread(()->{if(generation==discoveryGeneration&&scanning){for(Endpoint endpoint:snapshot){Boolean reachable=results.get(endpoint.name);if(reachable==null||!endpoints.containsKey(endpoint.name))continue;if(reachable)endpoint.misses=0;else if(++endpoint.misses>=2){android.util.Log.i("Relay","Removing unavailable computer "+endpoint.name);removeComputer(endpoint.name);}}}probingNearby=false;if(scanning)reachabilityHandler.postDelayed(reachabilityCheck,2500);});},"relay-nearby-poll").start();}
    @Override protected void onNewIntent(Intent i){super.onNewIntent(i); receive(i);}
    private void receive(Intent i){if(i.getData()!=null){Toast.makeText(this,"Use Relay’s scanner to verify the QR directly from your PC",Toast.LENGTH_LONG).show();}}
    private void startSharing(){
        if(ShareService.sharing||ShareService.connecting){refreshSession();return;}
        if(checkingConnection)return;
        final Uri target;
        try{target=Uri.parse(link.getText().toString().trim());if(!RelayConnection.validTarget(target))throw new Exception();}
        catch(Exception error){discoveryStatus.setText("Scan a fresh pairing QR from the PC screen.");return;}
        checkingConnection=true;discoveryStatus.setTextColor(muted);discoveryStatus.setText("Checking connection to your computer…");
        new Thread(()->{
            try(Socket probe=new Socket()){
                android.util.Log.i("Relay","Checking TCP route to "+target.getHost()+":"+target.getPort());
                probe.connect(new InetSocketAddress(target.getHost(),target.getPort()),3500);
                runOnUiThread(()->{checkingConnection=false;pending=target.toString();
                    if(requestedDirection==1){Uri withQuality=target.buildUpon().clearQuery().appendQueryParameter("v","6").appendQueryParameter("q",String.valueOf(selectedQuality)).appendQueryParameter("remember",target.getQueryParameter("remember")==null?"0":target.getQueryParameter("remember")).appendQueryParameter("computer",selectedComputer).build();Intent viewer=new Intent(this,PcViewerActivity.class).putExtra("link",withQuality.toString()).putExtra("receiverName",selectedComputer).putExtra("deviceName",deviceName);startActivityForResult(viewer,45);}
                    else{discoveryStatus.setText("Approve Android sharing; approve on your PC if requested.");requestSharingPermission();}
                });
            }catch(Exception error){android.util.Log.w("Relay","Computer is unreachable before screen permission");runOnUiThread(()->{checkingConnection=false;discoveryStatus.setTextColor(Color.rgb(195,63,76));discoveryStatus.setText("Cannot reach this computer over Wi-Fi. Reconnect Wi-Fi on both devices, or try a phone hotspot. Then select the computer again.");});}
        },"relay-connection-check").start();
    }
    private void requestSharingPermission(){
        if(shareAudio.isChecked()&&checkSelfPermission(Manifest.permission.RECORD_AUDIO)!=PackageManager.PERMISSION_GRANTED){requestPermissions(new String[]{Manifest.permission.RECORD_AUDIO},44);return;}
        launchCapturePermission();
    }
    private void launchCapturePermission(){MediaProjectionManager manager=(MediaProjectionManager)getSystemService(MEDIA_PROJECTION_SERVICE);Intent capture=Build.VERSION.SDK_INT>=34?manager.createScreenCaptureIntent(android.media.projection.MediaProjectionConfig.createConfigForDefaultDisplay()):manager.createScreenCaptureIntent();startActivityForResult(capture,42);}
    @Override public void onRequestPermissionsResult(int request,String[] permissions,int[] results){super.onRequestPermissionsResult(request,permissions,results);if(request==44){if(results.length==0||results[0]!=PackageManager.PERMISSION_GRANTED){shareAudio.setChecked(false);}launchCapturePermission();}}
    @Override protected void onSaveInstanceState(Bundle state){super.onSaveInstanceState(state);}
    @Override protected void onActivityResult(int request,int result,Intent data){super.onActivityResult(request,result,data);if(request==45){showDetection();if(data!=null&&data.hasExtra("error"))discoveryStatus.setText(data.getStringExtra("error"));return;}if(request==43&&result==RESULT_OK&&data!=null){String scanned=data.getStringExtra("connection");if(scanned!=null&&RelayConnection.validQr(Uri.parse(scanned)))confirmFirstPair(scanned);else Toast.makeText(this,"This QR is not a current Relay pairing code",Toast.LENGTH_LONG).show();return;} if(request==42&&result==RESULT_OK&&data!=null&&pending!=null){stopService(new Intent(this,ShareService.class)); Intent service=new Intent(this,ShareService.class).putExtra("link",pending).putExtra("result",result).putExtra("data",data).putExtra("shareAudio",shareAudio.isChecked()).putExtra("receiverName",selectedComputer).putExtra("deviceName",deviceName).putExtra("quality",selectedQuality); startForegroundService(service);}}
}
