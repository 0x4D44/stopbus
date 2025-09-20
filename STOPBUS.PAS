PROGRAM StopTheBus;

(* PROGRAM DESCRIPTION
*  ===================
*  CREATION - v1.0 - This is a card game - Stop the Bus - general Windows
*  3.1 program. Written and compiled using Borland's Turbo Pascal for
*  Windows v1.5, with use of the MicroSoft Help/Resource compilers. Thanks
*  to MicroSoft for the card bitmaps... 
*  Released 12/06/1993.
*
*  FIRST REVISION - v1.0a - Corrected scoring for a StopTheBus (ie score
*  of 31) as before only the player with the lowest score lost a life. Now
*  all players (apart from the one who stopped the bus) lose a life. Also
*  prevented users clicking on cards and moving them when a game is not in
*  progress. Updated resource file to reflect v1.0a status.
*  Released 22/06/1993.
*
*  SECOND REVISION - v1.0b - Corrected stack saving position bug. Also
*  modified to potentially use BWCC. Released 20/08/1993.
*
*  THIRD REVISION - v1.1 - Tidied up prior to Shareware release using CTL3D
*  and removing the annoying multi-menu options. These have been replaced
*  by a OPTIONS dialog box.
*  Released 01/05/1994.
*
*  FOURTH REVISION - v1.11 - How embarassing a minor intialisation bug caused
*  a few problems (!). Anyway it should be fixed now. Interestingly enough this
*  bug has been in from the start. Hmmm.
*  Released 09/05/1994.
*
*
*  PROGRAM INFROMATION
*  ===================
*  Author   : M G Davidson
*  Date     : 09/05/1994
*  Version  : 1.11
*  Language : PASCAL
*
*)

(* Library's used *)
USES WinTypes, WinProcs, WObjects, StdDlgs, WinDos, Strings, Ctl3d;

CONST
  CM_GAMEDEAL  = 100;    (* GAME|DEAL menu option *)
  CM_GAMEOPTN  = 101;    (* GAME|OPTIONS menu option *)
  CM_GAMEEXIT  = 102;    (* GAME|EXIT menu option *)
  CM_HELPCTNT  = 900;    (* HELP|CONTENTS menu option *)
  CM_HELPUSIN  = 901;    (* HELP|USING HELP menu option *)
  CM_HELPABOT  = 999;    (* HELP|ABOUT menu option *)
  ID_HELP      = 998;    (* General Help ID for dialogs etc *)
  ID_OPTNCARDS = 500;    (* Options dialog cheat cards check box *)
  ID_OPTNSTACK = 501;    (* Options dialog cheat deck check box *)
  ID_OPTNSCORE = 502;    (* Options dialog cheat scores check box *)
  ID_OPTNSEXIT = 503;    (* Options dialog save on exit check box *)
  ID_ABOTTITLE = 550;    (* About dialog program title box *)
  ID_COPYRIGHT = 551;    (* About dialog CopyRight static text control *)
  ID_REGNAME   = 552;    (* Licensed to name box *)
  ID_REGCOMP   = 553;    (* Licensed to company box *)
  ID_VERSIONNO = 554;    (* About dialog version number *)                       
  ID_RELSEDATE = 555;    (* About dialog release date *)
  ID_ADDRESS   = 556;    (* About dialog address *)
  ID_STICKBUT  = 1000;   (* Main window stick button *)
  ID_DEALBUT   = 1001;   (* Main window deal button *)
  ID_QUITBUT   = 1002;   (* Main window quit button *)
  ID_HAPPBUT   = 1003;   (* Main window "happy" button *)

  (* Other things *)
  CardWidth       = 71;  (* Card width *)
  CardHeight      = 96;  (* Card height *)
  SmallCardWidth  = 41;  (* Mini card width *)
  SmallCardHeight = 55;  (* Mini card height *)
  StackXPos       = 10;  (* Stack X position *)
  StackYPos       = 40;  (* Stack Y position *)

  (* Help contexts *)
  HC_OPTIONS  = 1;                     (* Options dialog help context *) 

  (* Ini and help file names *)
  INIFILENAME  = 'STOPBUS.INI';        (* Initialisation file name *)
  HELPFILENAME = 'STOPBUS.HLP';        (* Help file name *)

  (* Windows messages *)
  WM_START    = WM_USER + 1;           (* Start the game *)
  WM_JUSTCASE = WM_USER + 2;           (* Just in case of problems message *)

  (* Licence information *)
  UserName = 'This program is ShareWare.';                  
  CompanyName = 'If you like it then send £5 (UK Sterling) or $10 (US dollars) to:';
  Address = 'Martin G Davidson'+#13+
            'Hertford College'+#13+
            'Oxford'+#13+
            'OX1 3BW'+#13+
            'United Kingdom';


TYPE
  (* Playing card record *)
  TCard = RECORD
    LButDown: BOOLEAN;
    Card,
    XDefault,        
    YDefault: INTEGER;
  END;


  (* Main application type *)
  TCardApp = OBJECT(TApplication)
    PROCEDURE InitMainWindow; virtual;
  END;


  (* Options dialog prototype *)
  POptions = ^TOptions;
  TOptions = OBJECT(TDialog)
    Chk1, Chk2, Chk3, Chk4: PCheckBox;         (* Handles to four check boxes *)

    CONSTRUCTOR Init (AParent: PWindowsObject; AName: PChar);
    PROCEDURE   SetupWindow; virtual;
    PROCEDURE   OK(VAR Msg: TMessage);
                virtual ID_FIRST + ID_OK;
    PROCEDURE   Help(VAR Msg: TMessage);
                virtual ID_FIRST + ID_HELP;
  END;


  (* About box dialog prototype *)
  PAboutBox = ^TAboutBox;
  TAboutBox = OBJECT(TDialog)
    Text1, Text2, Text3, Text4,                
    Text5, Text6, Text7: PStatic;               (* Static text control handles *)

    CONSTRUCTOR Init(AParent: PWindowsObject; AName: PChar);
    PROCEDURE   SetupWindow; virtual;
  END;
                                      

  (* Cheat Cards window - displays other players cards *)
  PCheatCards = ^TCheatCards;
  TCheatCards = OBJECT(TWindow)

    CONSTRUCTOR Init(AParent: PWindowsObject);
    DESTRUCTOR  Done; virtual;
    PROCEDURE   GetWindowClass(VAR AWndClass: TWNDCLASS);
                virtual;
    PROCEDURE   SetupWindow; virtual;
    PROCEDURE   DrawScreen(TheDC: HDC);
    PROCEDURE   Paint(PaintDC: HDC; VAR PaintInfo: TPaintStruct);
                virtual;
    PROCEDURE   WMMove(VAR Msg: TMessage);
                virtual WM_FIRST + WM_MOVE;
    FUNCTION    GetClassName: PChar; virtual;
  END;


  (* Cheat Score window - displays other players current scores *)
  PCheatScores = ^TCheatScores;
  TCheatScores = OBJECT(TWindow)

    CONSTRUCTOR Init(AParent: PWindowsObject);
    DESTRUCTOR  Done; virtual;
    PROCEDURE   GetWindowClass(VAR AWndClass: TWNDCLASS);
                virtual;
    PROCEDURE   SetupWindow; virtual;
    PROCEDURE   DrawScreen(TheDC: HDC);
    PROCEDURE   Paint(PaintDC: HDC; VAR PaintInfo: TPaintStruct);
                virtual;
    PROCEDURE   WMMove(VAR Msg: TMessage);
                virtual WM_FIRST + WM_MOVE;
    FUNCTION    GetClassName: PChar; virtual;
  END;


  (* Cheat Stack window - displays next four cards in the stack of cards *)
  PCheatStack = ^TCheatStack;
  TCheatStack = OBJECT(TWindow)

    CONSTRUCTOR Init(AParent: PWindowsObject);
    DESTRUCTOR  Done; virtual;
    PROCEDURE   GetWindowClass(VAR AWndClass: TWNDCLASS);
                virtual;
    PROCEDURE   SetupWindow; virtual;
    PROCEDURE   DrawScreen(TheDC: HDC);
    PROCEDURE   Paint(PaintDC: HDC; VAR PaintInfo: TPaintStruct);
                virtual;
    PROCEDURE   WMMove(VAR Msg: TMessage);
                virtual WM_FIRST + WM_MOVE;
    FUNCTION    GetClassName: PChar; virtual;
  END;



  (* This is the main window prototype *)
  PMainWindow = ^TMainWindow;
  TMainWindow = OBJECT(TWindow)
    PlayerToStart, StickPlayer,                (* Current player information *)
    PreviousPlayer, OldStackCard: INTEGER;     (* More player information *)  
    Lives: ARRAY [1..4] OF INTEGER;            (* Player remaining lives array *)
    StopBus: INTEGER;                          (* Has (and who) someone stopped the bus? *)
    SaveExit: BOOLEAN;                         (* Should we save on exit? *)
    Playing: BOOLEAN;                          (* Which player is currently playing *)
    NotUsedNextCard, StackLButDown: BOOLEAN;   (* More logicals *)
    TGreenBrush: TLogBrush;                    (* Logical brush record *)
    GreenBrush: HBrush;                        (* Handle to a green brush *)
    StickBut, DealBut,                         (* <Stick> and <Deal> button handles *)
    QuitBut, HappBut: PButton;                 (* <Quit> and <Happy> button handles *)
    PntrChCa: PCheatCards;                     (* Handle to cheat cards window *) 
    PntrChSc: PCheatScores;                    (* Handle to cheat scores window *)
    PntrChSt: PCheatStack;                     (* Handle to cheat stack window *)
    ChCaX, ChCaY: INTEGER;                     (* Cheat cards window position *)
    ChStX, ChStY: INTEGER;                     (* Cheat stack window position *)
    ChScX, ChScY: INTEGER;                     (* Cheat score window position *)

    CONSTRUCTOR Init(AParent: PWindowsObject; ATitle: PChar);
    DESTRUCTOR  Done; virtual;
    PROCEDURE   GetWindowClass (VAR AWndClass: TWNDCLASS);
                virtual;
    PROCEDURE   SetupWindow; virtual;
    PROCEDURE   CMGameDeal(VAR Msg: TMessage);
                virtual CM_FIRST + CM_GAMEDEAL;
    PROCEDURE   CMGameOptn(VAR Msg: TMessage);
                virtual CM_FIRST + CM_GAMEOPTN;
    PROCEDURE   CMGameExit(VAR Msg: TMessage);
                virtual CM_FIRST + CM_GAMEEXIT;
    PROCEDURE   HelpCtnt(VAR Msg: TMessage);
                virtual CM_FIRST + CM_HELPCTNT;
    PROCEDURE   HelpUsin(VAR Msg: TMessage);
                virtual CM_FIRST + CM_HELPUSIN;
    PROCEDURE   HelpAbot(VAR Msg: TMessage);
                virtual CM_FIRST + CM_HELPABOT;
    PROCEDURE   Paint(PaintDC: HDC; VAR PaintInfo : TPaintStruct); virtual;
    PROCEDURE   WMLButtonDown(VAR Msg: TMessage);
                virtual WM_FIRST + WM_LBUTTONDOWN;
    PROCEDURE   WMStart(VAR Msg: TMessage);
                virtual WM_FIRST + WM_START;
    PROCEDURE   WMJustCase(VAR Msg: TMessage);
                virtual WM_FIRST + WM_JUSTCASE;
    PROCEDURE   WMLButtonUp(VAR Msg: TMessage);
                virtual WM_FIRST + WM_LBUTTONUP;
    PROCEDURE   PR_StickBut(VAR Msg: TMessage);
                virtual ID_FIRST + ID_STICKBUT;
    PROCEDURE   PR_DealBut(VAR Msg: TMessage);
                virtual ID_FIRST + ID_DEALBUT;
    PROCEDURE   PR_QuitBut(VAR Msg: TMessage);
                virtual ID_FIRST + ID_QUITBUT;
    PROCEDURE   PR_HappBut(VAR Msg: TMessage);
                virtual ID_FIRST + ID_HAPPBUT;
    PROCEDURE   HandleChCa(CreateWin: BOOL);
    PROCEDURE   HandleChSt(CreateWin: BOOL);
    PROCEDURE   HandleChSc(CreateWin: BOOL);
    PROCEDURE   DrawBMP(DC: HDC; X, Y, BitMap: HBitmap);
    PROCEDURE   Shuffle;
    PROCEDURE   Deal;
    PROCEDURE   GameControl;
    PROCEDURE   SuccessMessage(Person: INTEGER);
    PROCEDURE   StartGame;
    PROCEDURE   StartRound;
    PROCEDURE   DisplStrtPlayer(TheDC: HDC);
    PROCEDURE   Happ;
    PROCEDURE   AutoPlay(PlayerNo: INTEGER);
    PROCEDURE   AutoPlayOld(PlayerNo: INTEGER);
    PROCEDURE   SetPosInd(TheDC: HDC);
    PROCEDURE   Stick(TheDC: HDC);
    PROCEDURE   DisplScores(TheDC: HDC);
    PROCEDURE   Scores;
    FUNCTION    GetClassName: PChar; virtual;
    FUNCTION    MinScore: INTEGER;
    FUNCTION    ValScore(Value: INTEGER): INTEGER;
    FUNCTION    Suit(Value: INTEGER): INTEGER;
    FUNCTION    StopTheBus(WhichPlayer: INTEGER): BOOLEAN;
    FUNCTION    GetScore(PlayCard1, PlayCard2, PlayCard3: INTEGER): INTEGER;
  END;


VAR
  StackPntr: INTEGER;
  PlayerToPlay: INTEGER;
  PackCard: ARRAY [1..52] OF INTEGER;
  RoundScore: ARRAY [1..4] OF INTEGER;
  Player: ARRAY [1..4] OF ARRAY [1..3] OF TCard;
  CardCross: HBitmap;
  Cards: ARRAY [1..52] OF HBitmap;
  CardBack: HBitmap;


(*****************************************************************)
(* Methods of TOptions from here onwards....*)


CONSTRUCTOR TOptions.Init(AParent: PWindowsObject; AName: PChar);
BEGIN
  (* Call ancestor method *)
  TDialog.Init(AParent, AName);

  (* Get handles to controls *)
  Chk1 := New(PCheckBox, InitResource(@self, ID_OPTNCARDS));
  Chk2 := New(PCheckBox, InitResource(@self, ID_OPTNSTACK));
  Chk3 := New(PCheckBox, InitResource(@self, ID_OPTNSCORE));
  Chk4 := New(PCheckBox, InitResource(@self, ID_OPTNSEXIT));
END;


PROCEDURE TOptions.SetupWindow;
BEGIN
  (* Call ancestor method *)
  TDialog.SetupWindow;

  (* Set up controls *)
  IF (PMainWindow(Parent)^.PntrChCa <> NIL) THEN  Chk1^.SetCheck(BF_CHECKED);
  IF (PMainWindow(Parent)^.PntrChSt <> NIL) THEN  Chk2^.SetCheck(BF_CHECKED);
  IF (PMainWindow(Parent)^.PntrChSc <> NIL) THEN  Chk3^.SetCheck(BF_CHECKED);
  IF (PMainWindow(Parent)^.SaveExit) THEN  Chk4^.SetCheck(BF_CHECKED);
END;


PROCEDURE TOptions.OK(VAR Msg: TMessage);
BEGIN
  (* Update parent fields *)
  PMainWindow(Parent)^.HandleChCa(BOOL(Chk1^.GetCheck = BF_CHECKED));
  PMainWindow(Parent)^.HandleChSt(BOOL(Chk2^.GetCheck = BF_CHECKED));
  PMainWindow(Parent)^.HandleChSc(BOOL(Chk3^.GetCheck = BF_CHECKED));
  PMainWindow(Parent)^.SaveExit := BOOL(Chk4^.GetCheck = BF_CHECKED);

  (* Call ancestor method *)
  TDialog.OK(Msg);
END;


PROCEDURE TOptions.Help(VAR Msg: TMessage);
BEGIN
  (* Display relevant help information *)
  WinHelp(HWindow, HELPFILENAME, HELP_CONTEXT, HC_OPTIONS);
END;


(************************************************************************)
(* Methods of TAboutBox follow... *)


CONSTRUCTOR TAboutBox.Init(AParent: PWindowsObject; AName: PChar);
BEGIN
  (* Call ancesor method *)
  TDialog.Init(AParent, AName);

  (* Get handles to controls *)                              
  Text1 := New(PStatic, InitResource(@self, ID_ABOTTITLE, 15));
  Text2 := New(PStatic, InitResource(@self, ID_COPYRIGHT, 15));
  Text3 := New(PStatic, InitResource(@self, ID_REGNAME, 15));
  Text4 := New(PStatic, InitResource(@self, ID_REGCOMP, 15));
  Text5 := New(PStatic, InitResource(@self, ID_VERSIONNO, 15));
  Text6 := New(PStatic, InitResource(@self, ID_RELSEDATE, 15));
  Text7 := New(PStatic, InitResource(@self, ID_ADDRESS, 100));
END;
                                          

PROCEDURE TAboutBox.SetupWindow;
VAR
  TextString: ARRAY [0..100] OF CHAR;
BEGIN
  (* Call ancestor method *)
  TDialog.SetupWindow;

  (* Set window text *)
  SetWindowText(HWindow, 'About Stop the Bus');
                               
  (* Set controls appropriately *)
  Text1^.SetText('Stop the Bus');
  Text2^.SetText('Copyright © Martin Davidson - 1994');
  Text3^.SetText(UserName);
  Text4^.SetText(CompanyName);
  Text5^.SetText('Version 1.11');
  Text6^.SetText('Released: 09/05/1994');
  Text7^.SetText(Address);
END;


(*****************************************************************)
(* Methods of TMainWindow from here onwards....*)


CONSTRUCTOR TMainWindow.Init(AParent: PWindowsObject; ATitle: PChar);
VAR
  I: INTEGER;
BEGIN
  (* Call ancestor method *)
  TWindow.Init(AParent, ATitle);

  (* Object creation *)
  WITH TGreenBrush DO
  BEGIN
    lbStyle := BS_SOLID;
    lbColor := $00008000;
    lbHatch := 0;
  END;
  GreenBrush := CreateBrushIndirect(TGreenBrush);

  (* Window attributes *)
  Attr.W := 600;
  Attr.H := 400;
  Attr.X := GetPrivateProfileInt('Main Window', 'X position', 0, INIFILENAME);
  Attr.Y := GetPrivateProfileInt('Main Window', 'Y position', 0, INIFILENAME);
  Attr.Menu := LoadMenu(HInstance, 'Menu');
  Attr.Style := Attr.Style AND NOT(WS_MAXIMIZEBOX) AND NOT(WS_SIZEBOX);

  (* Load card bitmaps *)
  FOR I := 1 TO 52 DO
    Cards[I] := LoadBitmap(HInstance, PChar(I));

  (* Load card back bitmaps *)
  CardBack := LoadBitmap(HInstance, PChar(53));
  CardCross := LoadBitmap(HInstance, PChar(54));

  (* Setup playmode buttons *)
  StickBut := New(PButton, Init(@self, ID_StickBut, 'Stick',
                                400, 200, 50, 50, FALSE));
  DealBut := New(PButton, Init(@self, ID_DealBut, 'Deal',
                               500, 200, 50, 50, FALSE));
  QuitBut := New(PButton, Init(@self, ID_QuitBut, 'Exit',
                               500, 260, 50, 50, FALSE));
  HappBut := New(PButton, Init(@self, ID_HappBut, 'OK',
                               400, 260, 50, 50, TRUE));

  (* Program initialisation *)
  RANDOMIZE;
  StackPntr := 1;
  PlayerToPlay := 0;
  PlayerToStart := 0;
  PreviousPlayer := 1;
  StickPlayer := 0;
  FOR I := 1 TO 4 DO  RoundScore[I] := 0;
  FOR I := 1 TO 4 DO  Lives[I] := 3; 

  (* Initialise variables *)
  PntrChCa := NIL;
  PntrChSt := NIL;
  PntrChSc := NIL;
END;


DESTRUCTOR TMainWindow.Done;
VAR
  I: INTEGER;
  TextString: ARRAY [0..10] OF CHAR;
BEGIN
  (* Check to see if it should update initialisation file *)
  IF (SaveExit) THEN
  BEGIN
    (* Do cheat windows exist? *)
    Str(ORD(BOOL(PntrChCa <> NIL)), TextString);
    WritePrivateProfileString('Cheat Cards', 'Exists', TextString, INIFILENAME);
    Str(ORD(BOOL(PntrChSt <> NIL)), TextString);
    WritePrivateProfileString('Cheat Stack', 'Exists', TextString, INIFILENAME);
    Str(ORD(BOOL(PntrChSc <> NIL)), TextString);
    WritePrivateProfileString('Cheat Score', 'Exists', TextString, INIFILENAME);

    (* Cheat window positions *)
    Str(ChCaX, TextString);
    WritePrivateProfileString('Cheat Cards', 'X position', TextString, INIFILENAME);
    Str(ChCaY, TextString);
    WritePrivateProfileString('Cheat Cards', 'Y position', TextString, INIFILENAME);
    Str(ChStX, TextString);
    WritePrivateProfileString('Cheat Stack', 'X position', TextString, INIFILENAME);
    Str(ChStY, TextString);
    WritePrivateProfileString('Cheat Stack', 'Y position', TextString, INIFILENAME);
    Str(ChScX, TextString);
    WritePrivateProfileString('Cheat Score', 'X position', TextString, INIFILENAME);
    Str(ChScY, TextString);
    WritePrivateProfileString('Cheat Score', 'Y position', TextString, INIFILENAME);

    (* Main window position and size *)
    Str(Attr.X, TextString);
    WritePrivateProfileString('Main Window', 'X position', TextString, INIFILENAME);
    Str(Attr.Y, TextString);
    WritePrivateProfileString('Main Window', 'Y position', TextString, INIFILENAME);

    WritePrivateProfileString('Main Window', 'Save on exit', '1', INIFILENAME);
  END
  ELSE
    WritePrivateProfileString('Main Window', 'Save on exit', '0', INIFILENAME);

  (* Close help window *)
  WinHelp(HWindow, HELPFILENAME, HELP_QUIT, 0);

  (* Delete brush object *)
  DeleteObject(GreenBrush);

  (* Delete card objects *)
  FOR I := 1 TO 52 DO
    DeleteObject(Cards[I]);

  DeleteObject(CardBack);
  DeleteObject(CardCross);

  (* Call ancestor method *)
  TWindow.Done;
END;


PROCEDURE TMainWindow.GetWindowClass(VAR AWndClass: TWNDCLASS);
BEGIN
  (* Window particulars *)
  TWindow.GetWindowClass(AWndClass);
  AWndClass.hIcon := LoadIcon(HInstance, 'ICON_1');
  AWndClass.hbrBackGround := GreenBrush;
END;


PROCEDURE TMainWindow.SetupWindow;
VAR
  I, J: INTEGER;
  SentMessage: TMsg;
BEGIN
  (* Call ancestor method *)
  TWindow.SetupWindow;

  (* Set up cheat windows *)
  ChCaX := GetPrivateProfileInt('Cheat Cards', 'X position', 0, INIFILENAME);
  ChCaY := GetPrivateProfileInt('Cheat Cards', 'Y position', 280, INIFILENAME);
  ChStX := GetPrivateProfileInt('Cheat Stack', 'X position', 280, INIFILENAME);
  ChStY := GetPrivateProfileInt('Cheat Stack', 'Y position', 370, INIFILENAME);
  ChScX := GetPrivateProfileInt('Cheat Score', 'X position', 490, INIFILENAME);
  ChScY := GetPrivateProfileInt('Cheat Score', 'Y position', 370, INIFILENAME);

  SaveExit := BOOL(GetPrivateProfileInt('Main Window', 'Save on exit', 0, INIFILENAME));
  HandleChCa(BOOL(GetPrivateProfileInt('Cheat Cards', 'Exists', 0, INIFILENAME)));
  HandleChSt(BOOL(GetPrivateProfileInt('Cheat Stack', 'Exists', 0, INIFILENAME)));
  HandleChSc(BOOL(GetPrivateProfileInt('Cheat Score', 'Exists', 0, INIFILENAME)));

  (* Initialise storage record *)
  FOR I := 1 TO 4 DO
  BEGIN
    FOR J := 1 TO 3 DO
    BEGIN
      WITH Player[I, J] DO
      BEGIN
        LButDown := FALSE;
        Card := 0;
        XDefault := 0;
        YDefault := 0;
      END;
    END;
  END;

  (* Reintialise player 1's storage record *) 
  FOR J := 1 TO 3 DO
  BEGIN
    WITH Player[1, J] DO
    BEGIN
      XDefault := (100*J-90);
      YDefault := 200;
    END;
  END;

  (* Program initialisation *)
  EnableWindow(StickBut^.HWindow, FALSE);
  EnableWindow(HappBut^.HWindow, FALSE);

  (* Start game off - sort of *)
  PostMessage(HWindow, WM_JUSTCASE, 0, 0);
END;


PROCEDURE TMainWindow.CMGameDeal(VAR Msg: TMessage);
BEGIN
  (* Start game off *)
  StartGame;
  StartRound;
  GameControl;
END;


PROCEDURE TMainWindow.CMGameOptn(VAR Msg: TMessage);
BEGIN
  (* Display the <options> dialog box *)
  Application^.ExecDialog(New(POptions, Init(@self, 'OPTIONS')));
END;


PROCEDURE TMainWindow.CMGameExit(VAR Msg: TMessage);
BEGIN
  (* End the program *)
  TMainWindow.Done;
END;


PROCEDURE TMainWindow.HelpCtnt(VAR Msg: TMessage);
BEGIN
  (* Show help file contents *)
  WinHelp (HWindow, HELPFILENAME, HELP_INDEX, 0);
END;


PROCEDURE TMainWindow.HelpUsin(VAR Msg: TMessage);
BEGIN
  (* Show help on using help file contents *)
  WinHelp (HWindow, HELPFILENAME, HELP_HELPONHELP, 0);
END;


PROCEDURE TMainWindow.HelpAbot(VAR Msg: TMessage);
BEGIN
  (* Bring up about box *)
  Application^.ExecDialog(New(PAboutBox, Init(@self, 'AboutBox')));
END;


PROCEDURE TMainWindow.Paint(PaintDC: HDC; VAR PaintInfo: TPaintStruct);
VAR
  I: INTEGER;
  RectOne: TRect;
  TextString: ARRAY [0..80] OF CHAR;
BEGIN
  (* Call ancestor method *)
  TWindow.Paint(PaintDC, PaintInfo);

  (* Draw player 1's hand *)
  FOR I := 1 TO 3 DO
  BEGIN
    WITH Player[1, I] DO
      IF (Card > 0) THEN
      BEGIN
        DrawBMP(PaintDC, XDefault, YDefault, Cards[Card]);
      END
      ELSE
      BEGIN
        DrawBMP(PaintDC, XDefault, YDefault, CardCross);
      END;
  END;

  (* Draw card on top of the pack *)
  IF ((StackPntr > 0) AND (StackPntr < 53)) THEN
  BEGIN
    IF (PackCard[StackPntr] > 0) THEN
      DrawBMP(PaintDC, 110, 40, Cards[PackCard[StackPntr]]);
  END;

  (* Draw stack card *)
  DrawBMP(PaintDC, StackXPos, StackYPos, CardBack);

  (* Write text labels *)
  SetBkColor(PaintDC, $00008000);
  SetTextColor(PaintDC, $00000000);
  StrCopy(TextString, 'Deck:');
  TextOut(PaintDC, 20, 15, TextString, StrLen(TextString));
  StrCopy(TextString, 'Your hand:');
  TextOut(PaintDC, 20, 170, TextString, StrLen(TextString));
  StrCopy(TextString, 'Remaining Lives:');
  TextOut(PaintDC, 400, 55, TextString, StrLen(TextString));
  StrCopy(TextString, 'Player 1 - ');
  TextOut(PaintDC, 400, 75, TextString, StrLen(TextString));
  StrCopy(TextString, 'Player 2 - ');
  TextOut(PaintDC, 400, 90, TextString, StrLen(TextString));
  StrCopy(TextString, 'Player 3 - ');
  TextOut(PaintDC, 400, 105, TextString, StrLen(TextString));
  StrCopy(TextString, 'Player 4 - ');
  TextOut(PaintDC, 400, 120, TextString, StrLen(TextString));

  (* Write player information *)
  DisplStrtPlayer(PaintDC);
  DisplScores(PaintDC);

  StrCopy(TextString, ' ->');
  IF ((PlayerToPlay < 1) OR (PlayerToPlay > 4)) THEN PlayerToPlay := 1;
  TextOut(PaintDC, 380, (60+15*PlayerToPlay), TextString, StrLen(TextString));

  IF ((StickPlayer < 0) OR (StickPlayer > 4)) THEN StickPlayer := 0;
  IF (StickPlayer > 0) THEN
  BEGIN
    StrCopy(TextString, '***');
    TextOut(PaintDC, 360, (60+15*StickPlayer), TextString, StrLen(TextString));
  END;

END;


PROCEDURE TMainWindow.WMLButtonDown(VAR Msg: TMessage);
VAR
  I: INTEGER;
  MseXPos,MseYPos: INTEGER;
  ReturnValue: LONGINT;
  Top, Bottom, Left, Right: INTEGER;
BEGIN
  (* If player one is not meant to be playing - ie Playing is FALSE ... *)
  IF NOT(Playing) THEN EXIT;

  (* Get mouse positions and store *)
  MseXPos := LOWORD(Msg.lParam);
  MseYPos := HIWORD(Msg.lParam);

  IF (PlayerToPlay = 1) THEN
  BEGIN
    (* Three possible card positions to choose from *)
    (* Possibility number I *)
    FOR I := 1 TO 3 DO
    BEGIN
      Top := Player[1,I].YDefault;
      Bottom := Player[1,I].YDefault + CardHeight;
      Left := Player[1,I].XDefault;
      Right := Player[1,I].XDefault + CardWidth;

      IF ((MseXPos < Right) AND (MseXPos > Left)) THEN
      BEGIN
        IF ((MseYPos < Bottom) AND (MseYPos > Top )) THEN
        BEGIN
          WITH Player[1,I] DO
          BEGIN
            LButDown := TRUE;
          END;
        END;
      END;
    END;

    (* Check to see if mouse is over the stack *)
    Top := StackYPos;
    Bottom := StackYPos + CardHeight;
    Left := StackXPos;
    Right := StackXPos + CardWidth;

    StackLButDown := FALSE;
    IF ((MseXPos < Right) AND (MseXPos > Left)) THEN
    BEGIN
      IF ((MseYPos < Bottom) AND (MseYPos > Top )) THEN
      BEGIN
        StackLButDown := TRUE;
      END;
    END;

  END;
END;


PROCEDURE TMainWindow.WMStart(VAR Msg: TMessage);
BEGIN
  (* Start the program off *)
  StartGame;
  StartRound;
  GameControl;
END;


PROCEDURE TMainWindow.WMJustCase(VAR Msg: TMessage);
BEGIN
  (* Start game off properly *)
  PostMessage(HWindow, WM_START, 0, 0);
END;


PROCEDURE TMainWindow.WMLButtonUp(VAR Msg: TMessage);
VAR
  I, J, MseXPos, MseYPos,
  Top, Bottom, Left, Right: INTEGER;
BEGIN
  (* If player one is not meant to be playing - ie Playing is FALSE ... *)
  IF NOT(Playing) THEN EXIT;

  (* Get mouse positions and store *)
  MseXPos := LOWORD(Msg.lParam);
  MseYPos := HIWORD(Msg.lParam);

  FOR I := 1 TO 3 DO
  BEGIN

    IF (Player[1,I].LButDown = TRUE) THEN
    BEGIN
      (* Check mouse pointer is still over the same card *)
      Top := Player[1, I].YDefault;
      Bottom := Player[1, I].YDefault + CardHeight;
      Left := Player[1, I].XDefault;
      Right := Player[1, I].XDefault + CardWidth;

      IF ((MseXPos < Right) AND (MseXPos > Left)) THEN
      BEGIN
        IF ((MseYPos < Bottom) AND (MseYPos > Top )) THEN
        BEGIN
          (* Swap selected card with the stack card *)
          J := PackCard[StackPntr];
          PackCard[StackPntr] := Player[1, I].Card;
          Player[1, I].Card := J;

          (* Redraw cards *)
          DrawBMP(0, Player[1, I].XDefault, Player[1, I].YDefault,
                  Cards[Player[1,I].Card]);
          DrawBMP(0,110,40,Cards[PackCard[StackPntr]]);

          (* Control stick button appropriately *)
          IF (OldStackCard = PackCard[StackPntr]) THEN
          BEGIN
            IF (StickPlayer = 0) THEN
              EnableWindow(StickBut^.HWindow, TRUE);
          END
          ELSE
          BEGIN
            EnableWindow(StickBut^.HWindow, FALSE);
          END;
        END;
      END;
    END;
  END;

  (* Check to see if mouse is still over the stack *)
  Top := StackYPos;
  Bottom := StackYPos + CardHeight;
  Left := StackXPos;
  Right := StackXPos + CardWidth;

  IF (StackLButDown) THEN
  BEGIN
    IF ((MseXPos < Right) AND (MseXPos > Left)) THEN
    BEGIN
      IF ((MseYPos < Bottom) AND (MseYPos > Top )) THEN
      BEGIN
        IF (NotUsedNextCard) THEN
        BEGIN
          IF (OldStackCard = PackCard[StackPntr]) THEN
          BEGIN
            (* Now HAS used the next card *)
            NotUsedNextCard := FALSE;
            EnableWindow(StickBut^.HWindow, FALSE);
            INC(StackPntr);
            IF (StackPntr > 52) THEN
              MessageBox(HWindow, 'Deck overflow',
                         'Stop the Bus', MB_ICONEXCLAMATION);

            (* Update stack box - if it exists *)
            IF (PntrChSt <> NIL) THEN
              PntrChSt^.DrawScreen(0);

            (* Redraw stack of cards *)
            DrawBMP(0, 110, 40, Cards[PackCard[StackPntr]]);
          END;
        END;
      END;
    END;
  END;

END;


PROCEDURE TMainWindow.PR_StickBut(VAR Msg: TMessage);
BEGIN
  (* Call sticking routinue *)
  Stick(0);
  
  (* Pass control to "happy" routinue *)
  Happ;
END;


PROCEDURE TMainWindow.PR_DealBut(VAR Msg: TMessage);
BEGIN
 (* Call original routinue *)
 CMGameDeal(Msg);
END;


PROCEDURE TMainWindow.PR_QuitBut(VAR Msg: TMessage);
BEGIN
  (* Call "done" routinue *)
  Done;
END;


PROCEDURE TMainWindow.PR_HappBut(VAR Msg: TMessage);
BEGIN
  (* Call "happy" routinue *)
  Happ;
END;


PROCEDURE TMainWindow.HandleChCa(CreateWin: BOOL);
BEGIN
  (* Create or destroy the Cheat Cards window *)
  IF (CreateWin) THEN
  BEGIN
    IF (PntrChCa = NIL) THEN
    BEGIN
      (* Create a new window *)
      PntrChCa := New(PCheatCards, Init(@self));
      Application^.MakeWindow(PntrChCa);
    END;
  END
  ELSE
  BEGIN
    IF (PntrChCa <> NIL) THEN
    BEGIN
      (* Destroy the current window *)
      PntrChCa^.Done;
      PntrChCa := NIL;
    END;
  END;
END;


PROCEDURE TMainWindow.HandleChSt(CreateWin: BOOL);
BEGIN
  (* Create or destroy the Cheat Deck/Stack window *)
  IF (CreateWin) THEN
  BEGIN
    IF (PntrChSt = NIL) THEN
    BEGIN
      (* Create a new window *)
      PntrChSt := New(PCheatStack, Init(@self));
      Application^.MakeWindow(PntrChSt);
    END;
  END
  ELSE
  BEGIN
    IF (PntrChSt <> NIL) THEN
    BEGIN
      (* Destroy the current window *)
      PntrChSt^.Done;
      PntrChSt := NIL;
    END;
  END;
END;


PROCEDURE TMainWindow.HandleChSc(CreateWin: BOOL);
BEGIN
  (* Create or destroy the Cheat Score window *)
  IF (CreateWin) THEN
  BEGIN
    IF (PntrChSc = NIL) THEN
    BEGIN
      (* Create a new window *)
      PntrChSc := New(PCheatScores, Init(@self));
      Application^.MakeWindow(PntrChSc);
    END;
  END
  ELSE
  BEGIN
    IF (PntrChSc <> NIL) THEN
    BEGIN
      (* Destroy the current window *)
      PntrChSc^.Done;
      PntrChSc := NIL;
    END;
  END;
END;


PROCEDURE TMainWindow.DrawBMP(DC: HDC; X, Y, BitMap: HBitmap);
VAR
  MemDC: HDC;
  MadeDC: BOOLEAN;
  OldBitmap: HBitmap;
BEGIN
  (* Check to see if passed a DC *)
  IF (DC = 0) THEN
  BEGIN
    (* Otherwise create new DC *)
    DC := GetDC(HWindow);
    MadeDC := TRUE;
  END
  ELSE
    MadeDC := FALSE;

  (* Guts of moving procedure *)
  MemDC := CreateCompatibleDC(DC);
  OldBitmap := SelectObject(MemDC, BitMap);
  BitBlt(DC, X, Y, CardWidth, CardHeight, MemDC, 0, 0, SRCCopy);
  SelectObject(MemDC, OldBitmap);
  DeleteDC(MemDC);
  IF (MadeDC) THEN ReleaseDC(HWindow, DC);
END;


PROCEDURE TMainWindow.Shuffle;
VAR
  CardPosA, CardPosB, Temp, X: INTEGER;
BEGIN
  (* Fill card array *)
  FOR X := 1 TO 52 DO
    PackCard[X] := X;

  (* Shuffle card array *)
  FOR X := 1 TO 100 DO
  BEGIN
    (* Generate a random position *)
    CardPosA := RANDOM(52) + 1;
    CardPosB := RANDOM(52) + 1;

    (* Swap the cards about *)
    Temp := PackCard[CardPosA];
    PackCard[CardPosA] := PackCard[CardPosB];
    PackCard[CardPosB] := Temp;
  END;
END;


PROCEDURE TMainWindow.Deal;
VAR
  I, J, K: INTEGER;
BEGIN
  (* Call shuffling routinue *)
  Shuffle;

  K := 1;
  FOR I := 1 TO 3 DO
  BEGIN
    FOR J := 1 TO 4 DO
    BEGIN
      IF (Lives[J] > 0) THEN
      BEGIN
        Player[J,I].Card := PackCard[K]; 
        INC(K);
      END
      ELSE
      BEGIN
        Player[J,I].Card := 0;
      END;
    END;
  END;

  (* Set pack pointer to the appropriate place *)
  StackPntr := K;
END;


PROCEDURE TMainWindow.GameControl;
VAR
  J, M, N, P: INTEGER;
  OldStickPlayer: INTEGER;
  TextString, TextStrtwo: ARRAY [0..30] OF CHAR;
LABEL
    1, 2;
BEGIN
  (* Control is handed here from either "Happ" or from "Deal" *)
  (* The "real" player is player 1 - the others are all "automatic" *)
1:
  (* Reset not used card variable *)
  NotUsedNextCard := TRUE;
  Playing := FALSE;
  IF (StickPlayer = 0) THEN
    EnableWindow(StickBut^.HWindow, TRUE);

  (* Update scorebox - if it exists *)
  IF (PntrChSc <> NIL) THEN
  BEGIN
    J := MinScore;
    PntrChSc^.DrawScreen(0);
  END;

  (* Update stack box - if it exists *)
  IF (PntrChSt <> NIL) THEN
    PntrChSt^.DrawScreen(0);

  (* Check for "Stop The Bus" *)
  FOR J := 1 TO 4 DO
  BEGIN
    IF (StopTheBus(J)) THEN
    BEGIN
      SuccessMessage(J);
      StopBus := J;
      GOTO 2;
    END;
  END;

  (* Check for a "stuck" player *)
  OldStickPlayer := StickPlayer;
  IF (StickPlayer > 0) THEN
  BEGIN
    CASE StickPlayer OF
      1: N := 4;
      2: N := 1;
      3: N := 2;
      4: N := 3;
    END;
  END
  ELSE
    N := 4;

  (* Control the automatic players *)
  M := PlayerToPlay;
  IF (M > 1) THEN
  BEGIN
    FOR PlayerToPlay := M TO N DO
    BEGIN
      IF (Lives[PlayerToPlay] > 0) THEN
      BEGIN
        SetPosInd(0);
        AutoPlay(PlayerToPlay);

        (* Update scorebox - if it exists *)
        IF (PntrChSc <> NIL) THEN
        BEGIN
          J := MinScore;
          PntrChSc^.DrawScreen(0);
        END;

        IF (StopTheBus(PlayerToPlay)) THEN
        BEGIN
          SuccessMessage(PlayerToPlay);
          StopBus := PlayerToPlay;
          GOTO 2;
        END;
      END;
    END;
    PlayerToPlay := 1;
  END;

  (* Check for general "stick" *)
  IF (OldStickPlayer > 0) THEN
  BEGIN
2:
    Scores;
    (* Check for a "rare" condition - thanks to Matthew for finding it... *)
    IF ((Lives[1]=0) AND (Lives[2]=0) AND (Lives[3]=0)
       AND (Lives[4]=0)) THEN
    BEGIN
      EnableWindow(StickBut^.HWindow, FALSE);
      EnableWindow(HappBut^.HWindow, FALSE);
      MessageBox(HWindow, 'A draw is declared.', 'Stop the Bus', MB_OK);
      MessageBox(HWindow, 'That was a RARE message - Well Done!',
                 'Stop the Bus', MB_ICONINFORMATION);
      EXIT;
    END;
    J := 0;
    IF ((Lives[1]=0) AND (Lives[2]=0) AND (Lives[3]=0)) THEN J := 4;
    IF ((Lives[1]=0) AND (Lives[2]=0) AND (Lives[4]=0)) THEN J := 3;
    IF ((Lives[1]=0) AND (Lives[3]=0) AND (Lives[4]=0)) THEN J := 2;
    IF ((Lives[2]=0) AND (Lives[3]=0) AND (Lives[4]=0)) THEN J := 1;
    IF (J > 0) THEN
    BEGIN
      EnableWindow(StickBut^.HWindow, FALSE);
      EnableWindow(HappBut^.HWindow, FALSE);
      IF (J = 1) THEN
        StrCopy(TextString, 'Congratulations - you''ve won! ')
      ELSE
      BEGIN
        StrCopy(TextString, 'Player ');
        Str(J, TextStrtwo);
        StrCat(TextString, TextStrtwo);
        StrCat(TextString, ' has won.');
      END;
      MessageBox(HWindow, TextString, 'Stop the Bus', MB_ICONINFORMATION);
      EXIT;
    END
    ELSE
      StartRound;
      GOTO 1;
  END
  ELSE
  BEGIN
    (* PlayerToPlay must now be equal to one *)
    IF (Lives[1] > 0) THEN
    BEGIN
      SetPosInd(0);
      OldStackCard := PackCard[StackPntr];
      Playing := TRUE;
      EXIT;
    END
    ELSE
    BEGIN
      PlayerToPlay := 2;
      GOTO 1;
    END;
  END;

END;


PROCEDURE TMainWindow.SuccessMessage(Person: INTEGER);
VAR
  TextString, TextStrtwo: ARRAY [0..30] OF CHAR;
BEGIN
  (* Display end of game messages - if someone has stopped the bus *)
  IF (Person = 1) THEN
  BEGIN
    StrCopy(TextString,'You''ve stopped the bus!');
  END
  ELSE
  BEGIN
    StrCopy(TextString,'Player ');
    Str(Person, TextStrtwo);
    StrCat(TextString, TextStrtwo);
    StrCat(TextString, ' has stopped the bus.');
  END;
  MessageBox(HWindow, TextString, 'Stop the Bus', MB_ICONEXCLAMATION);
END;


PROCEDURE TMainWindow.StartGame;
VAR
  I: INTEGER;
BEGIN
  (* Game initialisation code goes here *)

  (* Enable stick and "happy" buttons *)
  EnableWindow(StickBut^.HWindow, TRUE);
  EnableWindow(HappBut^.HWindow, TRUE);

  (* Reset lives *)
  FOR I := 1 TO 4 DO
    Lives[I] := 3;

  (* Initialise player to play/start *)
  PlayerToPlay := 0;
  PlayerToStart := 0;
END;


PROCEDURE TMainWindow.StartRound;
VAR
  TheDC: HDC;
  TextS: ARRAY [0..30] OF CHAR;
  I, X: INTEGER;
BEGIN
  (* Round initialisation code goes here *)

  (* Reset StopBus as no one has stopped it *)
  StopBus := 0;

  (* Reset round scores *)
  FOR I := 1 TO 4 DO
    RoundScore[I] := 0;


  (* Call main dealing procedure *)
  Deal;

  (* Draw cards *)
  TheDC := GetDC(HWindow);
    FOR I := 1 TO 3 DO
    BEGIN
      WITH Player[1, I] DO
        IF (Card > 0) THEN
        BEGIN
          DrawBMP(TheDC, XDefault, YDefault, Cards[Card]);
        END
        ELSE
        BEGIN
          DrawBMP(TheDC, XDefault, YDefault, CardCross);
        END;
    END;

    (* Draw card on top of the pack *)
    IF (StackPntr > 0) THEN
    BEGIN
      IF (PackCard[StackPntr] > 0) THEN
        DrawBMP(TheDC, 110, 40, Cards[PackCard[StackPntr]]);
    END;

    (* Clear previous pointer/stick indicator display *)
    SetBkColor(TheDC, $00008000);
    SetTextColor(TheDC, $00000000);
    StrCopy(TextS, '          ');
    FOR I := 1 TO 4 DO
      TextOut(TheDC, 360, (60+15*I), TextS, StrLen(TextS));

  ReleaseDC(HWindow, TheDC);

  (* If there is a CheatCard "ChCa" window open, update it *)
  IF (PntrChCa <> NIL) THEN
    PntrChCa^.DrawScreen(0);

  (* If there is a CheatScores "ChSc" window open, update it *)
  IF (PntrChSc <> NIL) THEN
    PntrChSc^.DrawScreen(0);

  (* Initialisation *)
  StickPlayer := 0;

  (* Display scores *)
  DisplScores(0);
 
  (* Setup round parameters *)
  X := 0;
  REPEAT
    INC(PlayerToStart);
    IF (PlayerToStart = 5) THEN
      PlayerToStart := 1;
    INC(X);
    IF (X > 5) THEN
      MessageBox(HWindow, 'Problems - 1', 'Stop the Bus',
                 MB_ICONEXCLAMATION);
  UNTIL (Lives[PlayerToStart] > 0);

  PlayerToPlay := PlayerToStart;
  SetPosInd(0);
  DisplStrtPlayer(0);
END;


PROCEDURE TMainWindow.DisplStrtPlayer(TheDC: HDC);
VAR
  MadeDC: BOOLEAN;
  TextString,TextStrtwo: ARRAY [0..30] OF CHAR;
BEGIN
  (* Check to see if passed a DC *)
  IF (TheDC = 0) THEN
  BEGIN
    (* Otherwise create new DC *)
    TheDC := GetDC(HWindow);
    MadeDC := TRUE;
  END
  ELSE
    MadeDC := FALSE;

  IF ((PlayerToPlay < 1) OR (PlayerToPlay > 4)) THEN PlayerToPlay := 1;

  (* The drawing procedure *)
  SetBkColor(TheDC, $00008000);
  SetTextColor(TheDC, $00000000);
  StrCopy(TextString, 'Player ');
  Str(PlayerToPlay, TextStrtwo);
  StrCat(TextString, TextStrtwo);
  StrCat(TextString, ' to start'); 
  TextOut(TheDC, 400, 150, TextString, StrLen(TextString));

  (* Tidy up *)
  IF (MadeDC) THEN ReleaseDC(HWindow, TheDC);
END;


PROCEDURE TMainWindow.Happ;
VAR
  Max: INTEGER;
BEGIN
  (* Update PlayerToPlay variable *)
  INC(PlayerToPlay);
  IF (PlayerToPlay = 5) THEN
    PlayerToPlay := 1;
  GameControl;
END;


PROCEDURE TMainWindow.AutoPlay(PlayerNo: INTEGER);
VAR
  A, B, C, K, L, M: INTEGER;
  ValStack: INTEGER;
  BaseScore: INTEGER;
  PosMaxScore, MaxScore: INTEGER;
  TempHand: ARRAY [0..3] OF INTEGER;
  TempSwap: INTEGER;
BEGIN
  (* This is the main automatic playing routinue - it makes a quality
     decision regarding the first card on the stack. If it can increase
     its score by more than five points by taking this card then it takes
     it - otherwise it increments the stack and tries again.

     However firstly it decides whether to stick or not... *)

  (* Get value of stack card *)
  ValStack := PackCard[StackPntr];

  (* Copy players hand into a temporary working array *)
  FOR A := 1 TO 3 DO
    TempHand[A] := Player[PlayerNo, A].Card;

  (* Calculate base score *)
  BaseScore := GetScore(TempHand[1], TempHand[2], TempHand[3]);
  MaxScore := BaseScore;

  (* Decide whether to stick or not *)
  IF ((BaseScore > 25) AND (StickPlayer = 0)) THEN
  BEGIN
    Stick(0);
  END
  ELSE
  BEGIN
    (* Swap card with each in the players hand - and then calculate
       score *)
    FOR A := 1 TO 3 DO
    BEGIN
      TempSwap := ValStack;
      ValStack := TempHand[A];
      TempHand[A] := TempSwap;
  
      (* Calculate score *)        
      B := GetScore(TempHand[1], TempHand[2], TempHand[3]);
      IF (B > MaxScore) THEN
      BEGIN
        MaxScore := B;
        PosMaxScore := A;
      END; 
    END;

    (* Check to see if the 5 points rule is obeyed *)
    IF ((MaxScore > BaseScore) AND (MaxScore > 6)) THEN
    BEGIN
      (* Permanently swap cards *)
      TempSwap := PackCard[StackPntr];
      PackCard[StackPntr] := Player[PlayerNo, PosMaxScore].Card;
      Player[PlayerNo, PosMaxScore].Card := TempSwap;

      (* Update screen display *)
      IF (PntrChCa <> NIL) THEN
        PntrChCa^.DrawScreen(0);
      DrawBMP(0, 110, 40, Cards[PackCard[StackPntr]]);
    END
    ELSE
    BEGIN
      (* Increment the stack *)
      INC(StackPntr);
      IF (StackPntr = 53) THEN
        MessageBox(HWindow, 'Deck overflow', 'Stop the Bus',
                   MB_ICONEXCLAMATION);

      (* Update stack box - if it exists *)
      IF (PntrChSt <> NIL) THEN
        PntrChSt^.DrawScreen(0);

      (* Update screen display - stack only *)
      DrawBMP(0, 110, 40, Cards[PackCard[StackPntr]]);

      (* Repeat previous operation *)
      (* Get value of stack card *)
      ValStack := PackCard[StackPntr];

      (* Copy players hand into a temporary working array *)
      FOR A := 1 TO 3 DO
        TempHand[A] := Player[PlayerNo, A].Card;

      (* Calculate base score *)
      BaseScore := GetScore(TempHand[1],TempHand[2],TempHand[3]);
      MaxScore := BaseScore;

      (* Swap card with each in the players hand - and then calculate
         score *)
      FOR A := 1 TO 3 DO
      BEGIN
        TempSwap := ValStack;
        ValStack := TempHand[A];
        TempHand[A] := TempSwap;
  
        (* Calculate score *)
        B := GetScore(TempHand[1],TempHand[2],TempHand[3]);
        IF (B > MaxScore) THEN
        BEGIN
          MaxScore := B;
          PosMaxScore := A;
        END; 
      END;

      (* Check to see if swapping cards is worthwhile *)
      IF (MaxScore > BaseScore) THEN
      BEGIN
        (* Permanently swap cards *)
        TempSwap := PackCard[StackPntr];
        PackCard[StackPntr] := Player[PlayerNo, PosMaxScore].Card;
        Player[PlayerNo, PosMaxScore].Card := TempSwap;
    
        (* Update screen display *)
        IF (PntrChCa <> NIL) THEN
          PntrChCa^.DrawScreen(0);
        DrawBMP(0, 110, 40, Cards[PackCard[StackPntr]]);
      END;
    END;
  END;
END;


PROCEDURE TMainWindow.AutoPlayOld(PlayerNo: INTEGER);
VAR
  SuitToGoFor: INTEGER;
  SuitScores: ARRAY [1..4] OF INTEGER;
  A, B, C: INTEGER;
  J, K, L, M: INTEGER;
  SwapTemp: INTEGER;
  Temp: ARRAY [1..4] OF INTEGER;
  MaxScore: INTEGER;
  Score: INTEGER;
  SuitofFirst: INTEGER;
  TextString: ARRAY [0..30] OF CHAR;
  TextStrtwo: ARRAY [0..30] OF CHAR;
  Long: LONGINT;
BEGIN
  (* This routine has been supplanted - MGD *)

  (* Slow things down a bit *)
{  FOR Long := 1 TO 1000000 DO;
  (* This procedure plays for an automatic player 
     It follows the following set of rules, which are applied
     in the following order to decide which suit to pick :-

      1) - three cards of one suit
      2) - two cards sum to 18 points or more
      3) - an A,K,Q,J,10,9 of one suit are to hand
      4) - two cards sum to 10 points or more

     If none of these conditions are satisfied the player doesn't
     choose a suit. *) 

  (* Initialise *)
  SuitToGoFor := 0;

  (* Calculate players score on a suit by suit basis *)
  FOR A := 1 TO 4 DO
  BEGIN
    SuitScores[A] := 0;
    FOR B := 1 TO 3 DO
    BEGIN
      IF (Suit(Player[PlayerNo, B].Card) = A) THEN
         SuitScores[A] := SuitScores[A] +
                          ValScore(Player[PlayerNo, B].Card);
    END;
  END;

  (* Check for three cards of one suit *)
  A := Suit(Player[PlayerNo, 1].Card);
  B := Suit(Player[PlayerNo, 2].Card);
  C := Suit(Player[PlayerNo, 3].Card);

  IF ((A = B) AND (B = C)) THEN
    SuitToGoFor := A;


  (* Check for two cards which sum to 18 points or more *)
  IF (SuitToGoFor = 0) THEN
  BEGIN
    FOR A := 1 TO 4 DO
    BEGIN
      IF (SuitScores[A] > 18) THEN SuitToGoFor := A;
    END;
  END;

  (* Check for an A,K,Q,J,10,9,8 of a suit *)
  IF (SuitToGoFor = 0) THEN
  BEGIN
    B := 0; (* B is treated as maximum single card value *)
    C := 0; (* C is treated as a marker for card B *)
    FOR A := 1 TO 3 DO
    BEGIN
      IF (ValScore(Player[PlayerNo, A].Card) > B) THEN
      BEGIN
        B := ValScore(Player[PlayerNo, A].Card);
        C := A;
      END;
    END;
    IF (B > 7) THEN
      SuitToGoFor := Suit(Player[PlayerNo, C].Card);
  END;

  (* Check for two cards which sum to 10 points or more *)
  IF (SuitToGoFor = 0) THEN
  BEGIN
    FOR A := 1 TO 4 DO
    BEGIN
      IF (SuitScores[A] > 10) THEN SuitToGoFor := A;
    END;
  END;

  Str(PlayerNo,TextString);
  Str(SuitToGoFor,TextStrtwo);
  StrCat(TextString, ' is the player number ');
  StrCat(TextStrtwo, ' is the suit to go for ');
  MessageBox(HWindow,TextString,TextStrtwo,mb_ok);

  (* Otherwise don't choose anything

     Now is the point where what to do has to be decided :-

     Obviously if a suit has already been chosen this is much
     easier - the only card combinations that do not lead to
     a suit being chosen are ones where all cards are different
     and none is greater than seven and pairings where the two
     cards do not add up to more than nine.

     The following routine firstly sorts out players who have chosen
     a suit before looking at the harder case of those who haven't.

     For those who haven't chosen a suit the order of "play" is as
     follows :-

     Check card on stop of stack to see if it is an A,K,Q,J,10,9,8. If
     it is then pick it up. Otherwise take the next card from the pile
     and check it for A,K,Q,J,10,9,8 status. If it is then take it.
     Otherwise check it to see if it is the same suit as a card in the
     hand, and if this is so, swap it with the lowest card out of the
     remaining cards. *)

  (* Sort out players who have chosen a suit *)
  IF (SuitToGoFor > 0) THEN
  BEGIN
    IF (Suit(PackCard[StackPntr]) <> SuitToGoFor) THEN
    BEGIN
      INC(StackPntr);
      IF (StackPntr = 53) THEN
      BEGIN
        MessageBox(HWindow,'Deck overflow','Stop The Bus',mb_IconExclamation);
        (* Update stack box - if it exists *)
        IF (BoxExistChSt) THEN
          PntrChSt^.DrawScreen(0);
      END
      ELSE
      BEGIN
        (* Update screen display - stack only *)
        DrawBMP(0, 110, 40, Cards[PackCard[StackPntr]]);
        MessageBox(HWindow,'Deck incremented','Message 1',mb_ok);
      END;
    END;

    (* Find lowest card in player's hand *)
    B := ValScore(Player[PlayerNo, 1].Card);
    IF (Suit(Player[PlayerNo, 1].Card) = SuitToGoFor) THEN
      B := B + 11;
    C := 1; (* Treat as "pointer" to the minimum card *)
    FOR A := 2 TO 3 DO
    BEGIN
      K := ValScore(Player[PlayerNo, A].Card);
      IF (Suit(Player[PlayerNo, A].Card) = SuitToGoFor) THEN
        K := K + 11;
      IF (K < B) THEN
      BEGIN
        B := K;
        C := A;
      END;
    END;

    (* Check to see if card on offer is higher *)
    A := ValScore(Player[PlayerNo, C].Card);
    B := ValScore(PackCard[StackPntr]);
    IF (B > A) THEN
    BEGIN
      SwapTemp := PackCard[StackPntr];
      PackCard[StackPntr] := Player[PlayerNo, C].Card;
      Player[PlayerNo, C].Card := SwapTemp;

      (* Update screen display *)
      IF (BoxExistChCa) THEN
        PntrChCa^.DrawScreen(0);
      DrawBMP(0, 110, 40, Cards[PackCard[StackPntr]]);
      MessageBox(HWindow,'Cards Swapped','Message 2',mb_ok);
    END;
  END
  ELSE
  BEGIN
    (* Now deal with those players who haven't chosen a suit *)

    (* Check current card on the stack to see if it is
       lower than <A,K,Q,J,10,9,8> *)
    IF (ValScore(PackCard[StackPntr]) <= 7) THEN
    BEGIN
      INC(StackPntr);
      IF (StackPntr > 53) THEN
      BEGIN
        MessageBox(HWindow,'Deck overflow','Stop The Bus',mb_IconExclamation);
        (* Update stack box - if it exists *)
        IF (BoxExistChSt) THEN
          PntrChSt^.DrawScreen(0);
      END
      ELSE
      BEGIN
        (* Update screen display - stack only*)
        DrawBMP(0, 110, 40, Cards[PackCard[StackPntr]]);
        MessageBox(HWindow,'Deck incremented','Message 5',mb_ok);
      END;
    END;

    IF (ValScore(PackCard[StackPntr]) > 7) THEN
    BEGIN
      (* Find lowest card in hand *)
      B := 12; (* Treat as actual value of minimum card *)
      C := 0;  (* Treat as "pointer" to minimum card *)
      FOR A := 1 TO 3 DO
      BEGIN
        IF (ValScore(Player[PlayerNo, A].Card) < B) THEN
        BEGIN
          B := ValScore(Player[PlayerNo, A].Card);
          C := A;
        END;
      END;

      (* Swap lowest card and new found card around *)
      SwapTemp := PackCard[StackPntr];
      PackCard[StackPntr] := Player[PlayerNo, C].Card;
      Player[PlayerNo, C].Card := SwapTemp;

      (* Update screen display *)
      IF (BoxExistChCa) THEN
        PntrChCa^.DrawScreen(0);
      DrawBMP(0, 110, 40, Cards[PackCard[StackPntr]]);
      MessageBox(HWindow,'Cards Swapped','Message 6',mb_ok);
    END
    ELSE
    BEGIN
      (* Check to see if the stack card is the same as one of the cards
         in the players hand *)
      B := 0; (* Found indicator *)
      K := 12; (* Lowest card magnitude holder *)
      L := 1; (* Treat as a "pointer" to the lowest card *) 
      FOR A := 1 TO 3 DO
      BEGIN
        IF (Suit(PackCard[StackPntr]) =
            Suit(Player[PlayerNo, A].Card)) THEN
        BEGIN
          B := 1;
        END
        ELSE
        BEGIN
          IF (ValScore(Player[PlayerNo, A].Card) < K) THEN
          BEGIN
            K := ValScore(Player[PlayerNo, A].Card);
            L := A;
          END;          
        END;
      END;
      IF (B = 1) THEN (* Suits match *)
      BEGIN
        (* Swap card L with value K with stack card *)
        SwapTemp := PackCard[StackPntr];
        PackCard[StackPntr] := Player[PlayerNo, L].Card;
        Player[PlayerNo, L].Card := SwapTemp;

        (* Update screen display *)
        IF (BoxExistChCa) THEN
          PntrChCa^.DrawScreen(0);
        DrawBMP(0, 110, 40, Cards[PackCard[StackPntr]]);
        MessageBox(HWindow,'Cards swapped','Message 7',mb_ok);
      END;
    END;
  END;

  (* Calculate score and decide whether to stick or not based on this *)
  IF (Lives[PlayerNo] > 0) THEN
  BEGIN
    MaxScore := 0;

    FOR J := 1 TO 3 DO
    BEGIN
      SuitofFirst := Suit(Player[PlayerNo,J].Card);
      Score := ValScore(Player[PlayerNo,J].Card);

      FOR K := 1 TO 2 DO
      BEGIN
        L := J + K;
        IF (L > 3) THEN
          L := L - 3;
        IF (Suit(Player[PlayerNo,L].Card) = SuitofFirst) THEN
        BEGIN
          Score := Score + ValScore(Player[PlayerNo,L].Card);
        END;
      END;

      (* Update player I's maximum possible score *)
      IF (Score > MaxScore) THEN
        MaxScore := Score;
    END;

  END;

  (* Decide whether to stick or not *)
  IF (MaxScore > 15) THEN
    Stick(0);
 }
END;


PROCEDURE TMainWindow.SetPosInd(TheDC: HDC);
VAR
  Temp: INTEGER;
  MadeDC: BOOLEAN;
  TextS: ARRAY [0..10] OF CHAR;
BEGIN
  (* Check to see if passed a DC *)
  IF (TheDC = 0) THEN
  BEGIN
    (* Otherwise create new DC *)
    TheDC := GetDC(HWindow);
    MadeDC := TRUE;
  END
  ELSE
    MadeDC := FALSE;

  (* This procedure sets the player position indicator *)
  SetBkColor(TheDC,$00008000);
  SetTextColor(TheDC,$00000000);
  StrCopy(TextS,'    ');
  TextOut(TheDC, 380, (60+15*PreviousPlayer), TextS, StrLen(TextS));
  StrCopy(TextS,' ->');
  TextOut(TheDC, 380, (60+15*PlayerToPlay), TextS, StrLen(TextS));
  PreviousPlayer := PlayerToPlay;

  (* Tidy up *)
  IF (MadeDC) THEN ReleaseDC(HWindow, TheDC);
END;


PROCEDURE TMainWindow.Stick(TheDC: HDC);
VAR
  MadeDC: BOOLEAN;
  TextS: ARRAY [0..10] OF CHAR;
BEGIN
  IF (StickPlayer = 0) THEN
  BEGIN
    (* Check to see if passed a DC *)
    IF (TheDC = 0) THEN
    BEGIN
      (* Otherwise create new DC *)
      TheDC := GetDC(HWindow);
      MadeDC := TRUE;
    END
    ELSE
      MadeDC := FALSE;

    (* This is the sticking routinue *)
    StickPlayer := PlayerToPlay;
    EnableWindow(StickBut^.HWindow, FALSE);

    (* Display stick indicator *)
    SetBkColor(TheDC, $00008000);
    SetTextColor(TheDC, $00000000);
    StrCopy(TextS, '***');
    TextOut(TheDC, 360, (60+15*PlayerToPlay), TextS, StrLen(TextS));

    (* Tidy up *)
    IF (MadeDC) THEN ReleaseDC(HWindow, TheDC);
  END;
END;


PROCEDURE TMainWindow.DisplScores(TheDC: HDC);
VAR
  I: INTEGER;
  MadeDC: BOOLEAN;
  TextS: ARRAY [0..30] OF CHAR;
BEGIN
  (* Check to see if passed a DC *)
  IF (TheDC = 0) THEN
  BEGIN
    (* Otherwise create new DC *)
    TheDC := GetDC(HWindow);
    MadeDC := TRUE;
  END
  ELSE
    MadeDC := FALSE;

  (* The drawing procedure *)
  SetBkColor(TheDC,$00008000);
  SetTextColor(TheDC,$00000000);
  FOR I := 1 TO 4 DO
  BEGIN
    Str(Lives[I], TextS);
    TextOut(TheDC, 470, (60+15*I), TextS, StrLen(TextS));
  END;

  (* Tidy up *)
  IF (MadeDC) THEN ReleaseDC(HWindow, TheDC);
END;


PROCEDURE TMainWindow.Scores;
VAR
  I, J: INTEGER;
  Play: ARRAY [0..3] OF INTEGER;
  LowestScore: INTEGER;
  TextString: ARRAY [0..30] OF CHAR;
  TextStrtwo: ARRAY [0..30] OF CHAR;
BEGIN
  (* Check to see if someone has stopped the bus *)
  IF (Stopbus > 0) THEN
  BEGIN
    J := 0;
    FOR I := 1 TO 4 DO
    BEGIN
      IF (Stopbus <> I) THEN
      BEGIN
        IF (Lives[I] > 0) THEN
        BEGIN
          Lives[I] := Lives[I] - 1;
          INC(J);
          Play[J] := I;
        END;
      END;
    END;
    CASE J OF
      1: BEGIN
           IF (Play[1] = 1) THEN
           BEGIN
             StrCopy(TextString, 'You''ve lost a life!');
           END
           ELSE
           BEGIN
             StrCopy(TextString, 'Player ');
             Str(Play[1], TextStrtwo);
             StrCat(TextString, TextStrtwo);
             StrCat(TextString, ' lost a life.');
           END;
         END;
      2: BEGIN
           StrCopy(TextString, 'Players ');
           Str(Play[1], TextStrtwo);
           StrCat(TextString, TextStrtwo);
           StrCat(TextString, ' & ');
           Str(Play[2], TextStrtwo);
           StrCat(TextString, TextStrtwo);
           StrCat(TextString, ' both lost a life.');
         END;
      3: BEGIN
           StrCopy(TextString, 'Players ');
           Str(Play[1], TextStrtwo);
           StrCat(TextString, TextStrtwo);
           StrCat(TextString, ', ');
           Str(Play[2], TextStrtwo);
           StrCat(TextString, TextStrtwo);
           StrCat(TextString, ' & ');
           Str(Play[3], TextStrtwo);
           StrCat(TextString, TextStrtwo);
           StrCat(TextString, ' all lost a life.');
         END;
    END;
    MessageBox(HWindow, TextString, 'Stop the Bus', MB_ICONINFORMATION);
  END
  ELSE
  BEGIN
    (* Call score calculating procedure *)
    LowestScore := MinScore;

    (* Update score board window - if it exists *)
    IF (PntrChSc <> NIL) THEN
      PntrChSc^.DrawScreen(0);

    (* Update lives left *)
    FOR I := 1 TO 4 DO
    IF (RoundScore[I] = LowestScore) THEN
    BEGIN
      Lives[I] := Lives[I] - 1;
      IF (Lives[I] = 0) THEN
      BEGIN
        IF (I = 1) THEN
        BEGIN
          MessageBox(HWindow,
                     'Unfortunately you''ve been knocked out!',
                     'Stop the Bus',
                     MB_ICONEXCLAMATION);
        END
        ELSE
        BEGIN
          StrCopy(TextString, 'Player ');
          Str(I, TextStrtwo);
          StrCat(TextString, TextStrtwo);
          StrCat(TextString, ' has no lives left.');
          MessageBox(HWindow, TextString, 'Stop the Bus', MB_ICONINFORMATION);
        END;
      END
      ELSE
      BEGIN
        (* Check to see who lost a life *)
        IF (I = 1) THEN
        BEGIN
          StrCopy(TextString, 'You''ve lost a life!');
        END
        ELSE
        BEGIN
          StrCopy(TextString, 'Player ');
          Str(I, TextStrtwo);
          StrCat(TextString, TextStrtwo);
          StrCat(TextString, ' lost a life.');
        END;
        MessageBox(HWindow, TextString, 'Stop the Bus', MB_ICONINFORMATION);
      END;
    END;
  END;

  (* Update displayed scores *)
  DisplScores(0);
END;


FUNCTION TMainWindow.GetClassName: PChar;
BEGIN
  GetClassName := 'Main Window';
END;


FUNCTION TMainWindow.MinScore: INTEGER;
VAR
  Temp: ARRAY [1..4] OF INTEGER;
  MaxScore: INTEGER;
  LowScore: INTEGER;
  Score: INTEGER;
  SuitofFirst: INTEGER;
  I, J, K, L: INTEGER;
BEGIN
  LowScore := 100;

  FOR I := 1 TO 4 DO
  BEGIN
    IF (Lives[I] > 0) THEN
    BEGIN
      MaxScore := 0;

      FOR J := 1 TO 3 DO
      BEGIN
        SuitofFirst := Suit(Player[I,J].Card);
        Score := ValScore(Player[I,J].Card);

        FOR K := 1 TO 2 DO
        BEGIN
          L := J + K;
          IF (L > 3) THEN
            L := L - 3;
          IF (Suit(Player[I,L].Card) = SuitofFirst) THEN
          BEGIN
            Score := Score + ValScore(Player[I,L].Card);
          END;
        END;

        (* Update player I's maximum possible score *)
        IF (Score > MaxScore) THEN
          MaxScore := Score;
      END;

      (* Update general score array *)
      RoundScore[I] := MaxScore;

      (* Update lowest score array *)
      IF (MaxScore < LowScore) THEN
      BEGIN
        LowScore := MaxScore;
      END;
    END;
  END;

  (* Set return value *)
  MinScore := LowScore;
END;

                           
FUNCTION TMainWindow.ValScore(Value: INTEGER): INTEGER;
VAR
  Temp: INTEGER;
BEGIN
  WHILE (Value > 13) DO
  BEGIN
    Value := Value - 13;
  END;

  CASE Value OF
    1: Temp := 11;
   11: Temp := 10;
   12: Temp := 10;
   13: Temp := 10;
  ELSE
       Temp := Value;
  END;

  (* Set return value *)
  ValScore := Temp;
END;


FUNCTION TMainWindow.Suit(Value: INTEGER): INTEGER;
VAR
  Temp: INTEGER;
BEGIN
  CASE Value OF
     1..13: Temp := 1;
    14..26: Temp := 2;
    27..39: Temp := 3;
    40..52: Temp := 4;
  END;

  (* Set return value *)
  Suit := Temp;
END;


FUNCTION TMainWindow.StopTheBus(WhichPlayer: INTEGER): BOOLEAN;
VAR
  I, X, Y, Z: INTEGER;
  StartSuit: INTEGER;
  Score: INTEGER;
BEGIN
  FOR X := 1 TO 3 DO
  BEGIN
    StartSuit := Suit(Player[WhichPlayer, X].Card);
    Score := ValScore(Player[WhichPlayer, X].Card); 
    FOR Y := 1 TO 2 DO
    BEGIN
      Z := X + Y;
      IF (Z > 3) THEN Z := Z - 3;
      IF (Suit(Player[WhichPlayer, Z].Card) = StartSuit) THEN
        Score :=  Score + ValScore(Player[WhichPlayer, Z].Card);
    END;
  END;

  (* Determine return value *)
  StopThebus := FALSE;
  IF (Score = 31) THEN
    StopThebus := TRUE;
END;


FUNCTION TMainWindow.GetScore(PlayCard1,PlayCard2,PlayCard3: INTEGER): INTEGER;
VAR
  PlayCard: ARRAY [1..3] OF INTEGER;
  MaxScore, Score: INTEGER;
  SuitofFirst: INTEGER;
  J, K, L: INTEGER;
BEGIN
  (* Copy passed values into array *)
  PlayCard[1] := PlayCard1;
  PlayCard[2] := PlayCard2;
  PlayCard[3] := PlayCard3;

  (* Calculate score *)
  MaxScore := 0;

  FOR J := 1 TO 3 DO
  BEGIN
    SuitofFirst := Suit(PlayCard[J]);
    Score := ValScore(PlayCard[J]);

    FOR K := 1 TO 2 DO
    BEGIN
      L := J + K;
      IF (L > 3) THEN
        L := L - 3;
      IF (Suit(PlayCard[L]) = SuitofFirst) THEN
      BEGIN
        Score := Score + ValScore(PlayCard[L]);
      END;
    END;

    (* Update player I's maximum possible score *)
    IF (Score > MaxScore) THEN
      MaxScore := Score;
  END;

  (* Set return value appropriately *)
  GetScore := MaxScore;
END;


(*****************************************************************)
(* Methods of TCheatCards follow... *)


CONSTRUCTOR TCheatCards.Init(AParent: PWindowsObject);
BEGIN
  (* Call ancestor method *)
  TWindow.Init(AParent, 'Cards Cheat');

  (* Set up window appropriately *) 
  WITH Attr DO
  BEGIN
    Style := Style OR WS_POPUP OR WS_CAPTION OR WS_SYSMENU;
    X := PMainWindow(Parent)^.ChCaX;
    Y := PMainWindow(Parent)^.ChCaY;
    W := 280;
    H := 200;
  END;
END;                                         


DESTRUCTOR TCheatCards.Done;
BEGIN
  (* Reset parent handle to window *)
  PMainWindow(Parent)^.PntrChCa := NIL;

  (* Call ancestor method *)
  TWindow.Done;
END;


PROCEDURE TCheatCards.GetWindowClass(VAR AWndClass: TWNDCLASS);
BEGIN
  (* Window particulars *)
  TWindow.GetWindowClass (AWndClass);
  AWndClass.hbrBackGround := GetStockObject(LTGRAY_BRUSH);
END;


PROCEDURE TCheatCards.SetupWindow;
BEGIN
  (* Call ancestor method *)
  TWindow.SetupWindow;

  (* Call screen drawing routinue *)
  DrawScreen(0);
END;


PROCEDURE TCheatCards.DrawScreen(TheDC: HDC);
VAR
  DC,MemDC: HDC;
  A, B: INTEGER;
  MadeDC: BOOLEAN;
  TextString, TextStrtwo: ARRAY [0..30] OF CHAR;
BEGIN

  (* Check to see if passed a DC *)
  IF (TheDC = 0) THEN
  BEGIN
    (* Otherwise create new DC *)
    TheDC := GetDC(HWindow);
    MadeDC := TRUE;
  END
  ELSE
    MadeDC := FALSE;

  (* Draw cards first *)
  MemDC := CreateCompatibleDC(TheDC);
  SelectObject(MemDC, Cards[2]);
  FOR A := 2 TO 4 DO
  BEGIN
    FOR B := 1 TO 3 DO
    BEGIN
      WITH Player[A,B] DO
      BEGIN
        IF (Card > 0) THEN
        BEGIN
          SelectObject(MemDC, Cards[Card]);
          StretchBlt(TheDC, (50+51*B), (57*A-108), SmallCardWidth,
                     SmallCardHeight, MemDC, 0, 0, CardWidth,
                     CardHeight, SRCCopy);
        END
        ELSE
        BEGIN
          SelectObject(MemDC, CardCross);
          StretchBlt(TheDC, (50+51*B), (57*A-108), SmallCardWidth,
                     SmallCardHeight, MemDC, 0, 0, CardWidth,
                     CardHeight, SRCCopy);
        END;
      END;
    END;
  END;
  DeleteDC(MemDC);

  (* Then draw text - Set up display *)
  SetBkColor(TheDC, RGB(192, 192, 192));
  SetTextColor(TheDC, RGB(0, 0, 0));

  FOR A := 2 TO 4 DO
  BEGIN
   StrCopy(TextString, 'Player ');
   Str(A, TextStrtwo);
   StrCat(TextString, TextStrtwo);
   StrCat(TextString, ': ');
   TextOut(TheDC, 10, (57*A-100), TextString, StrLen(TextString));
  END;

  (* Tidy up *)
  IF (MadeDC) THEN ReleaseDC(HWindow, TheDC);
END;


PROCEDURE TCheatCards.Paint(PaintDC: HDC; VAR PaintInfo: TPaintStruct);
BEGIN
  (* Call ancestor method *)
  TWindow.Paint(PaintDC, PaintInfo);

  (* Call screen drawing routinue *)
  DrawScreen(PaintDC);
END;


PROCEDURE TCheatCards.WMMove(VAR Msg: TMessage);
BEGIN
  PMainWindow(Parent)^.ChCaX := LOWORD(Msg.lParam) - 1;
  PMainWindow(Parent)^.ChCaY := HIWORD(Msg.lParam) - 20;
END;


FUNCTION TCheatCards.GetClassName: PChar;
BEGIN
  GetClassName := 'Card window';
END;


(*****************************************************************)
(* Methods of TCheatScores follow....*)


CONSTRUCTOR TCheatScores.Init(AParent: PWindowsObject);
BEGIN
  (* Call ancestor method *)
  TWindow.Init(AParent, 'Scores Cheat');

  (* Set up window appropriately *)
  WITH Attr DO
  BEGIN
    Style := Style OR WS_POPUP OR WS_CAPTION OR WS_SYSMENU;
    X := PMainWindow(Parent)^.ChScX;
    Y := PMainWindow(Parent)^.ChScY;
    W := 150;
    H := 110;
  END;
END;


DESTRUCTOR TCheatScores.Done;
BEGIN
  (* Reset parent handle to window *)
  PMainWindow(Parent)^.PntrChSc := NIL;

  (* Call ancestor method *)
  TWindow.Done;
END;


PROCEDURE TCheatScores.GetWindowClass(VAR AWndClass: TWNDCLASS);
BEGIN
  (* Window particulars *)
  TWindow.GetWindowClass (AWndClass);
  AWndClass.hbrBackGround := GetStockObject(LTGRAY_BRUSH);
END;


PROCEDURE TCheatScores.SetupWindow;
BEGIN
  (* Call ancestor method *)
  TWindow.SetupWindow;

  (* Call screen drawing routine *)
  DrawScreen(0);
END;


PROCEDURE TCheatScores.DrawScreen(TheDC: HDC);
VAR
  I: INTEGER;
  MadeDC: BOOLEAN;
  TextString, TextStrtwo: ARRAY [0..30] OF CHAR;
BEGIN
  (* Check to see if passed a DC *)
  IF (TheDC = 0) THEN
  BEGIN
    (* Otherwise create new DC *)
    TheDC := GetDC(HWindow);
    MadeDC := TRUE;
  END
  ELSE
    MadeDC := FALSE;

  (* Set up colours appropriately *)
  SetBkColor(TheDC, RGB(192, 192, 192));
  SetTextColor(TheDC, RGB(0, 0, 0));

  FOR I := 1 TO 4 DO
  BEGIN
    StrCopy(TextString, 'Player ');
    Str(I, TextStrtwo);
    StrCat(TextString, TextStrtwo);
    StrCat(TextString, ':     ');
    Str(RoundScore[I], TextStrtwo);
    StrCat(TextString, TextStrtwo);
    StrCat(TextString, '     ');
    TextOut(TheDC, 20, (20*I-16), TextString, StrLen(TextString));
  END;

  (* Tidy up *)
  IF (MadeDC) THEN ReleaseDC(HWindow, TheDC);                     
END;


PROCEDURE TCheatScores.Paint(PaintDC: HDC; VAR PaintInfo: TPaintStruct);
BEGIN
  (* Call screen drawing routinue *)
  DrawScreen(PaintDC);
END;


PROCEDURE TCheatScores.WMMove(VAR Msg: TMessage);
BEGIN
  PMainWindow(Parent)^.ChScX := LOWORD(Msg.lParam) - 1;
  PMainWindow(Parent)^.ChScY := HIWORD(Msg.lParam) - 20;
END;


FUNCTION TCheatScores.GetClassName: PChar;
BEGIN
  GetClassName := 'Score window';
END;


(*****************************************************************)
(* Methods of TCheatStack follow....*)


CONSTRUCTOR TCheatStack.Init(AParent: PWindowsObject);
BEGIN
  (* Call ancestor method *)
  TWindow.Init (AParent, 'Stack Cheat');

  (* Set up window appropriately *)
  WITH Attr DO
  BEGIN
    Style := Style OR WS_POPUP OR WS_CAPTION OR WS_SYSMENU;
    X := PMainWindow(Parent)^.ChStX;
    Y := PMainWindow(Parent)^.ChStY;
    W := 210;
    H := 110;
  END;
END;


DESTRUCTOR TCheatStack.Done;
BEGIN
  (* Reset parent handle to window *)
  PMainWindow(Parent)^.PntrChSt := NIL;

  (* Call ancestor method *)
  TWindow.Done;
END;


PROCEDURE TCheatStack.GetWindowClass(VAR AWndClass: TWNDCLASS);
BEGIN
  (* Window particulars *)
  TWindow.GetWindowClass (AWndClass);
  AWndClass.hbrBackGround := GetStockObject(LTGRAY_BRUSH);
END;


PROCEDURE TCheatStack.SetupWindow;
BEGIN
  (* Call ancestor method *)
  TWindow.SetupWindow;

  (* Call screen drawing routine *)
  DrawScreen(0);
END;


PROCEDURE TCheatStack.DrawScreen(TheDC: HDC);
VAR
  I, J: INTEGER;
  MadeDC: BOOLEAN;
  MemDC: HDC;
  TextString,TextStrtwo: ARRAY [0..30] OF CHAR;
  OldBitmap: HBitmap;
BEGIN
  (* Check to see if passed a DC *)
  IF (TheDC = 0) THEN
  BEGIN
    (* Otherwise create new DC *)
    TheDC := GetDC(HWindow);
    MadeDC := TRUE;
  END
  ELSE
    MadeDC := FALSE;

  (* Draw the stack cards *)
  MemDC := CreateCompatibleDC(TheDC);
  FOR I := 1 TO 4 DO
  BEGIN
    J := StackPntr + I;
    IF (J < 53) THEN
    BEGIN
      IF (J > 0) THEN
      BEGIN
        IF (PackCard[J] > 0) THEN
        BEGIN
          OldBitmap := SelectObject(MemDC, Cards[PackCard[J]]);
        END;
      END;
    END
    ELSE
    BEGIN
      OldBitmap := SelectObject(MemDC, CardCross);
    END;
    StretchBlt(TheDC, (50*I - 45), 30, SmallCardWidth,
               SmallCardHeight, MemDC, 0, 0, CardWidth,
               CardHeight, SRCCOPY);
  END;

  (* Set up colours appropriately *)
  SetBkColor(TheDC, RGB(192, 192, 192));
  SetTextColor(TheDC, RGB(0, 0, 0));

  StrCopy(TextString, 'Stack Pointer:  ');
  Str(StackPntr, TextStrtwo);
  StrCat(TextString, TextStrtwo);
  StrCat(TextString, '   ');
  TextOut(TheDC, 10, 5, TextString, StrLen(TextString));

  (* Reset old bitmap *)
  SelectObject(MemDC, OldBitmap);

  (* Tidy up *)
  DeleteDC(MemDC);
  IF (MadeDC) THEN ReleaseDC(HWindow, TheDC);
END;


PROCEDURE TCheatStack.Paint(PaintDC: HDC; VAR PaintInfo: TPaintStruct);
BEGIN
  (* Call screen drawing routinue *)
  DrawScreen(PaintDC);
END;


PROCEDURE TCheatStack.WMMove(VAR Msg: TMessage);
BEGIN
  PMainWindow(Parent)^.ChStX := LOWORD(Msg.lParam) - 1;
  PMainWindow(Parent)^.ChStY := HIWORD(Msg.lParam)- 20;
END;


FUNCTION TCheatStack.GetClassName: PChar;
BEGIN
  GetClassName := 'Stack window';
END;


(*****************************************************************)
(* Methods of TCardApp from here onwards....*)


PROCEDURE TCardApp.InitMainWindow;
BEGIN
  (* Define the main window *)
  MainWindow := New(PMainWindow, Init(nil, 'Stop the Bus'));
END;


(*****************************************************************)
(* Main Program....*)


VAR
  CardGame: TCardApp;

BEGIN
  (* Load custom controls *)
  Ctl3dRegister(HInstance);
  Ctl3dAutoSubclass(HInstance);

  CardGame.Init('STOPBUS');
  CardGame.Run;
  CardGame.Done;

  (* Unload custom controls *)
  Ctl3dUnRegister(HInstance);
END.