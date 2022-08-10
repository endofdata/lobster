#![allow(non_upper_case_globals)]
#![allow(dead_code)]

pub type SpeakerArrangement = i64;
pub type Speaker = i64;

pub const SPEAKER_L : Speaker = 1 << 0;
pub const SPEAKER_R : Speaker = 1 << 1;
pub const SPEAKER_C : Speaker = 1 << 2;
pub const SPEAKER_Lfe : Speaker = 1 << 3;
pub const SPEAKER_Ls : Speaker = 1 << 4;
pub const SPEAKER_Rs : Speaker = 1 << 5;
pub const SPEAKER_Lc : Speaker = 1 << 6;
pub const SPEAKER_Rc : Speaker = 1 << 7;
pub const SPEAKER_S : Speaker = 1 << 8;
pub const SPEAKER_Cs : Speaker = SPEAKER_S;
pub const SPEAKER_Sl : Speaker = 1 << 9;
pub const SPEAKER_Sr : Speaker = 1 << 10;
pub const SPEAKER_Tc : Speaker = 1 << 11;
pub const SPEAKER_Tfl : Speaker = 1 << 12;
pub const SPEAKER_Tfc : Speaker = 1 << 13;
pub const SPEAKER_Tfr : Speaker = 1 << 14;
pub const SPEAKER_Trl : Speaker = 1 << 15;
pub const SPEAKER_Trc : Speaker = 1 << 16;
pub const SPEAKER_Trr : Speaker = 1 << 17;
pub const SPEAKER_Lfe2 : Speaker = 1 << 18;
pub const SPEAKER_M : Speaker = 1 << 19;

pub const SPEAKER_ACN0 : Speaker = 1 << 20;
pub const SPEAKER_ACN1 : Speaker = 1 << 21;
pub const SPEAKER_ACN2 : Speaker = 1 << 22;
pub const SPEAKER_ACN3 : Speaker = 1 << 23;
pub const SPEAKER_ACN4 : Speaker = 1 << 38;
pub const SPEAKER_ACN5 : Speaker = 1 << 39;
pub const SPEAKER_ACN6 : Speaker = 1 << 40;
pub const SPEAKER_ACN7 : Speaker = 1 << 41;
pub const SPEAKER_ACN8 : Speaker = 1 << 42;
pub const SPEAKER_ACN9 : Speaker = 1 << 43;
pub const SPEAKER_ACN10 : Speaker = 1 << 44;
pub const SPEAKER_ACN11 : Speaker = 1 << 45;
pub const SPEAKER_ACN12 : Speaker = 1 << 46;
pub const SPEAKER_ACN13 : Speaker = 1 << 47;
pub const SPEAKER_ACN14 : Speaker = 1 << 48;
pub const SPEAKER_ACN15 : Speaker = 1 << 49;

pub const SPEAKER_Tsl : Speaker = 1 << 24;
pub const SPEAKER_Tsr : Speaker = 1 << 25;
pub const SPEAKER_Lcs : Speaker = 1 << 26;
pub const SPEAKER_Rcs : Speaker = 1 << 27;

pub const SPEAKER_Bfl : Speaker = 1 << 28;
pub const SPEAKER_Bfc : Speaker = 1 << 29;
pub const SPEAKER_Bfr : Speaker = 1 << 30;

pub const SPEAKER_Pl : Speaker = 1 << 31;
pub const SPEAKER_Pr : Speaker = 1 << 32;

pub const SPEAKER_Bsl : Speaker = 1 << 33;
pub const SPEAKER_Bsr : Speaker = 1 << 34;
pub const SPEAKER_Brl : Speaker = 1 << 35;
pub const SPEAKER_Brc : Speaker = 1 << 36;
pub const SPEAKER_Brr : Speaker = 1 << 37;



pub const SPARR_Empty : SpeakerArrangement = 0;
pub const SPARR_Mono : SpeakerArrangement = SPEAKER_M;
pub const SPARR_Stereo : SpeakerArrangement = SPEAKER_L   | SPEAKER_R;
pub const SPARR_StereoSurround : SpeakerArrangement = SPEAKER_Ls  | SPEAKER_Rs;
pub const SPARR_StereoCenter : SpeakerArrangement = SPEAKER_Lc  | SPEAKER_Rc;
pub const SPARR_StereoSide : SpeakerArrangement = SPEAKER_Sl  | SPEAKER_Sr;
pub const SPARR_StereoCLfe : SpeakerArrangement = SPEAKER_C   | SPEAKER_Lfe;
pub const SPARR_StereoTF : SpeakerArrangement = SPEAKER_Tfl | SPEAKER_Tfr;
pub const SPARR_StereoTS : SpeakerArrangement = SPEAKER_Tsl | SPEAKER_Tsr;
pub const SPARR_StereoTR : SpeakerArrangement = SPEAKER_Trl | SPEAKER_Trr;
pub const SPARR_StereoBF : SpeakerArrangement = SPEAKER_Bfl | SPEAKER_Bfr;
pub const SPARR_CineFront : SpeakerArrangement = SPEAKER_L   | SPEAKER_R | SPEAKER_C | SPEAKER_Lc | SPEAKER_Rc;

/** L R C */
pub const SPARR_30Cine : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C;
/** L R C Lfe */
pub const SPARR_31Cine : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C   | SPEAKER_Lfe;
/** L R S */
pub const SPARR_30Music : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_Cs;
/** L R Lfe S */
pub const SPARR_31Music : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_Lfe | SPEAKER_Cs;
/** L R C   S (LCRS) */
pub const SPARR_40Cine : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C   | SPEAKER_Cs;
/** L R C   Lfe S (LCRS+Lfe) */
pub const SPARR_41Cine : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C   | SPEAKER_Lfe | SPEAKER_Cs;
/** L R Ls  Rs (Quadro) */
pub const SPARR_40Music : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_Ls  | SPEAKER_Rs;
/** L R Lfe Ls Rs (Quadro+Lfe) */
pub const SPARR_41Music : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_Lfe | SPEAKER_Ls  | SPEAKER_Rs;
/** L R C   Ls Rs */									// 5.0 (ITU 0+5+0.0 Sound System B)
pub const SPARR_50 : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C   | SPEAKER_Ls  | SPEAKER_Rs;
/** L R C  Lfe Ls Rs */									// 5.1 (ITU 0+5+0.1 Sound System B)
pub const SPARR_51 : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C   | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs;
/** L R C  Ls  Rs Cs */
pub const SPARR_60Cine : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C   | SPEAKER_Ls  | SPEAKER_Rs | SPEAKER_Cs;
/** L R C  Lfe Ls Rs Cs */
pub const SPARR_61Cine : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C   | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Cs;
/** L R Ls Rs  Sl Sr */
pub const SPARR_60Music : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_Ls  | SPEAKER_Rs  | SPEAKER_Sl | SPEAKER_Sr;
/** L R Lfe Ls  Rs Sl Sr */
pub const SPARR_61Music : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_Lfe | SPEAKER_Ls  | SPEAKER_Rs | SPEAKER_Sl | SPEAKER_Sr;
/** L R C   Ls  Rs Lc Rc */
pub const SPARR_70Cine : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C   | SPEAKER_Ls  | SPEAKER_Rs | SPEAKER_Lc | SPEAKER_Rc;
/** L R C Lfe Ls Rs Lc Rc */
pub const SPARR_71Cine : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C   | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Lc | SPEAKER_Rc;
pub const SPARR_71CineFullFront : SpeakerArrangement = SPARR_71Cine;
/** L R C   Ls  Rs Sl Sr */								// (ITU 0+7+0.0 Sound System I)
pub const SPARR_70Music : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C   | SPEAKER_Ls  | SPEAKER_Rs | SPEAKER_Sl | SPEAKER_Sr;
/** L R C Lfe Ls Rs Sl Sr */							// (ITU 0+7+0.1 Sound System I)
pub const SPARR_71Music : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C   | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Sl | SPEAKER_Sr;

/** L R C Lfe Ls Rs Lcs Rcs */
pub const SPARR_71CineFullRear : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C   | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Lcs | SPEAKER_Rcs;
pub const SPARR_71CineSideFill : SpeakerArrangement = SPARR_71Music;
/** L R C Lfe Ls Rs Pl Pr */
pub const SPARR_71Proximity : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C   | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Pl | SPEAKER_Pr;

/** L R C Ls  Rs Lc Rc Cs */
pub const SPARR_80Cine : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C   | SPEAKER_Ls  | SPEAKER_Rs | SPEAKER_Lc | SPEAKER_Rc | SPEAKER_Cs;
/** L R C Lfe Ls Rs Lc Rc Cs */
pub const SPARR_81Cine : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C   | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Lc | SPEAKER_Rc | SPEAKER_Cs;
/** L R C Ls  Rs Cs Sl Sr */
pub const SPARR_80Music : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C   | SPEAKER_Ls  | SPEAKER_Rs | SPEAKER_Cs | SPEAKER_Sl | SPEAKER_Sr;
/** L R C Lfe Ls Rs Cs Sl Sr */
pub const SPARR_81Music : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C   | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Cs | SPEAKER_Sl | SPEAKER_Sr;
/** L R C Ls Rs Lc Rc Sl Sr */
pub const SPARR_90Cine : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C   | SPEAKER_Ls  | SPEAKER_Rs | SPEAKER_Lc | SPEAKER_Rc |
                                           SPEAKER_Sl | SPEAKER_Sr;
/** L R C Lfe Ls Rs Lc Rc Sl Sr */
pub const SPARR_91Cine : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C   | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Lc | SPEAKER_Rc |
                                           SPEAKER_Sl | SPEAKER_Sr;
/** L R C Ls Rs Lc Rc Cs Sl Sr */
pub const SPARR_100Cine : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C   | SPEAKER_Ls  | SPEAKER_Rs | SPEAKER_Lc | SPEAKER_Rc | SPEAKER_Cs |
                                           SPEAKER_Sl | SPEAKER_Sr;
/** L R C Lfe Ls Rs Lc Rc Cs Sl Sr */
pub const SPARR_101Cine : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C   | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Lc | SPEAKER_Rc | SPEAKER_Cs |
                                           SPEAKER_Sl | SPEAKER_Sr;

/** First-Order with Ambisonic Channel Number (ACN) ordering and SN3D normalization */
pub const SPARR_Ambi1stOrderACN : SpeakerArrangement = SPEAKER_ACN0 | SPEAKER_ACN1 | SPEAKER_ACN2 | SPEAKER_ACN3;
/** Second-Order with Ambisonic Channel Number (ACN) ordering and SN3D normalization */
pub const SPARR_Ambi2cdOrderACN : SpeakerArrangement = SPARR_Ambi1stOrderACN | SPEAKER_ACN4 | SPEAKER_ACN5 | SPEAKER_ACN6 | SPEAKER_ACN7 | SPEAKER_ACN8;
/** Third-Order with Ambisonic Channel Number (ACN) ordering and SN3D normalization */
pub const SPARR_Ambi3rdOrderACN : SpeakerArrangement = SPARR_Ambi2cdOrderACN | SPEAKER_ACN9 | SPEAKER_ACN10 | SPEAKER_ACN11 | SPEAKER_ACN12 | SPEAKER_ACN13 | SPEAKER_ACN14 | SPEAKER_ACN15;


/*-----------*/
/* 3D formats */
/*-----------*/
/** L R Ls Rs Tfl Tfr Trl Trr */						// 4.0.4
pub const SPARR_80Cube : SpeakerArrangement = SPEAKER_L | SPEAKER_R | SPEAKER_Ls | SPEAKER_Rs  | SPEAKER_Tfl| SPEAKER_Tfr| SPEAKER_Trl | SPEAKER_Trr;
pub const SPARR_40_4 : SpeakerArrangement = SPARR_80Cube;

/** L R C Lfe Ls Rs Cs Tc */							// 6.1.1
pub const SPARR_71CineTopCenter : SpeakerArrangement = SPEAKER_L | SPEAKER_R | SPEAKER_C  | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Cs  | SPEAKER_Tc; 

/** L R C Lfe Ls Rs Cs Tfc */							// 6.1.1
pub const SPARR_71CineCenterHigh : SpeakerArrangement = SPEAKER_L | SPEAKER_R | SPEAKER_C  | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Cs  | SPEAKER_Tfc; 

/** L R C Ls Rs Tfl Tfr */								// 5.0.2 (ITU 2+5+0.0 Sound System C)
pub const SPARR_70CineFrontHigh : SpeakerArrangement = SPEAKER_L | SPEAKER_R | SPEAKER_C | SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Tfl | SPEAKER_Tfr;
pub const SPARR_70MPEG3D : SpeakerArrangement = SPARR_70CineFrontHigh;
pub const SPARR_50_2 : SpeakerArrangement = SPARR_70CineFrontHigh;

/** L R C Lfe Ls Rs Tfl Tfr */							// 5.1.2 (ITU 2+5+0.1 Sound System C)
pub const SPARR_71CineFrontHigh : SpeakerArrangement = SPEAKER_L | SPEAKER_R | SPEAKER_C | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Tfl | SPEAKER_Tfr;
pub const SPARR_71MPEG3D : SpeakerArrangement = SPARR_71CineFrontHigh;
pub const SPARR_51_2 : SpeakerArrangement = SPARR_71CineFrontHigh;

/** L R C Lfe Ls Rs Tsl Tsr */							// 5.1.2 (Side)
pub const SPARR_71CineSideHigh : SpeakerArrangement = SPEAKER_L | SPEAKER_R | SPEAKER_C  | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Tsl | SPEAKER_Tsr; 

/** L R Lfe Ls Rs Tfl Tfc Tfr Bfc */					// 4.1.3.1
pub const SPARR_81MPEG3D : SpeakerArrangement = SPEAKER_L | SPEAKER_R | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs |
                                           SPEAKER_Tfl | SPEAKER_Tfc | SPEAKER_Tfr | SPEAKER_Bfc;
pub const SPARR_41_4_1 : SpeakerArrangement = SPARR_81MPEG3D;

/** L R C Ls Rs Tfl Tfr Trl Trr */						// 5.0.4 (ITU 4+5+0.0 Sound System D)
pub const SPARR_90 : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C | SPEAKER_Ls  | SPEAKER_Rs |
                                           SPEAKER_Tfl| SPEAKER_Tfr | SPEAKER_Trl | SPEAKER_Trr;
pub const SPARR_50_4 : SpeakerArrangement = 90;

/** L R C Lfe Ls Rs Tfl Tfr Trl Trr */					// 5.1.4 (ITU 4+5+0.1 Sound System D)
pub const SPARR_91 : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs  |
                                           SPEAKER_Tfl| SPEAKER_Tfr | SPEAKER_Trl | SPEAKER_Trr;
pub const SPARR_51_4 : SpeakerArrangement = 91;

/** L R C Ls Rs Tfl Tfr Trl Trr Bfc */					// 5.0.4.1 (ITU 4+5+1.0 Sound System E)
pub const SPARR_50_4_1 : SpeakerArrangement = SPEAKER_L | SPEAKER_R | SPEAKER_C | SPEAKER_Ls | SPEAKER_Rs |
										   SPEAKER_Tfl | SPEAKER_Tfr | SPEAKER_Trl | SPEAKER_Trr | SPEAKER_Bfc;

/** L R C Lfe Ls Rs Tfl Tfr Trl Trr Bfc */					// 5.1.4.1 (ITU 4+5+1.1 Sound System E)
pub const SPARR_51_4_1 : SpeakerArrangement = SPEAKER_L | SPEAKER_R | SPEAKER_C | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs |
										   SPEAKER_Tfl | SPEAKER_Tfr | SPEAKER_Trl | SPEAKER_Trr | SPEAKER_Bfc;

/** L R C Ls Rs Sl Sr Tsl Tsr */						// 7.0.2
pub const SPARR_70_2 : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C | SPEAKER_Ls | SPEAKER_Rs | 
                                           SPEAKER_Sl | SPEAKER_Sr | SPEAKER_Tsl | SPEAKER_Tsr;

/** L R C Lfe Ls Rs Sl Sr Tsl Tsr */					// 7.1.2
pub const SPARR_71_2 : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs | 
                                           SPEAKER_Sl | SPEAKER_Sr | SPEAKER_Tsl | SPEAKER_Tsr;
pub const SPARR_91Atmos : SpeakerArrangement = 71_2;		// 9.1 Dolby Atmos (3D)

/** L R C Ls Rs Sl Sr Tsl Tsr Trc */					// 7.0.3 (ITU 3+7+0.0 Sound System F)
pub const SPARR_70_3 : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C | SPEAKER_Ls | SPEAKER_Rs | 
                                           SPEAKER_Sl | SPEAKER_Sr | SPEAKER_Tsl | SPEAKER_Tsr | SPEAKER_Trc;

/** L R C Lfe Ls Rs Sl Sr Tsl Tsr Trc Lfe2 */			// 7.2.3 (ITU 3+7+0.2 Sound System F)
pub const SPARR_72_3 : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs | 
                                           SPEAKER_Sl | SPEAKER_Sr | SPEAKER_Tsl | SPEAKER_Tsr | SPEAKER_Trc | SPEAKER_Lfe2;

/** L R C Ls Rs Sl Sr Tfl Tfr Trl Trr */				// 7.0.4 (ITU 4+7+0.0 Sound System J)
pub const SPARR_70_4 : SpeakerArrangement = SPEAKER_L | SPEAKER_R | SPEAKER_C | SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Sl | SPEAKER_Sr |
                                           SPEAKER_Tfl | SPEAKER_Tfr | SPEAKER_Trl | SPEAKER_Trr;

/** L R C Lfe Ls Rs Sl Sr Tfl Tfr Trl Trr */			// 7.1.4 (ITU 4+7+0.1 Sound System J)
pub const SPARR_71_4 : SpeakerArrangement = SPEAKER_L | SPEAKER_R | SPEAKER_C | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Sl | SPEAKER_Sr |
                                           SPEAKER_Tfl | SPEAKER_Tfr | SPEAKER_Trl | SPEAKER_Trr;
pub const SPARR_111MPEG3D : SpeakerArrangement = 71_4;

/** L R C Ls Rs Sl Sr Tfl Tfr Trl Trr Tsl Tsr */		// 7.0.6
pub const SPARR_70_6 : SpeakerArrangement = SPEAKER_L | SPEAKER_R | SPEAKER_C | 
                                           SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Sl | SPEAKER_Sr | 
                                           SPEAKER_Tfl | SPEAKER_Tfr | SPEAKER_Trl | SPEAKER_Trr | SPEAKER_Tsl | SPEAKER_Tsr;

/** L R C Lfe Ls Rs Sl Sr Tfl Tfr Trl Trr Tsl Tsr */	// 7.1.6
pub const SPARR_71_6 : SpeakerArrangement = SPEAKER_L | SPEAKER_R | SPEAKER_C | SPEAKER_Lfe | 
                                           SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Sl | SPEAKER_Sr | 
                                           SPEAKER_Tfl | SPEAKER_Tfr | SPEAKER_Trl | SPEAKER_Trr | SPEAKER_Tsl | SPEAKER_Tsr;

/** L R C Ls Rs Lc Rc Sl Sr Tfl Tfr Trl Trr */			// 9.0.4 (ITU 4+9+0.0 Sound System G)
pub const SPARR_90_4 : SpeakerArrangement = SPEAKER_L | SPEAKER_R | SPEAKER_C |
                                           SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Lc | SPEAKER_Rc | SPEAKER_Sl | SPEAKER_Sr |
                                           SPEAKER_Tfl | SPEAKER_Tfr | SPEAKER_Trl | SPEAKER_Trr;

/** L R C Lfe Ls Rs Lc Rc Sl Sr Tfl Tfr Trl Trr */		// 9.1.4 (ITU 4+9+0.1 Sound System G)
pub const SPARR_91_4 : SpeakerArrangement = SPEAKER_L | SPEAKER_R | SPEAKER_C | SPEAKER_Lfe | 
                                           SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Lc | SPEAKER_Rc | SPEAKER_Sl | SPEAKER_Sr |
                                           SPEAKER_Tfl | SPEAKER_Tfr | SPEAKER_Trl | SPEAKER_Trr;

/** L R C Lfe Ls Rs Lc Rc Sl Sr Tfl Tfr Trl Trr Tsl Tsr */ // 9.0.6
pub const SPARR_90_6 : SpeakerArrangement = SPEAKER_L | SPEAKER_R | SPEAKER_C |
                                           SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Lc | SPEAKER_Rc | SPEAKER_Sl | SPEAKER_Sr |
                                           SPEAKER_Tfl | SPEAKER_Tfr | SPEAKER_Trl | SPEAKER_Trr | SPEAKER_Tsl | SPEAKER_Tsr;

/** L R C Lfe Ls Rs Lc Rc Sl Sr Tfl Tfr Trl Trr Tsl Tsr */ // 9.1.6
pub const SPARR_91_6 : SpeakerArrangement = SPEAKER_L | SPEAKER_R | SPEAKER_C | SPEAKER_Lfe | 
                                           SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Lc | SPEAKER_Rc | SPEAKER_Sl | SPEAKER_Sr |
                                           SPEAKER_Tfl | SPEAKER_Tfr | SPEAKER_Trl | SPEAKER_Trr | SPEAKER_Tsl | SPEAKER_Tsr;

/** L R C Ls Rs Tc Tfl Tfr Trl Trr */					// 5.0.5
pub const SPARR_100 : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C | SPEAKER_Ls  | SPEAKER_Rs | 
                                           SPEAKER_Tc | SPEAKER_Tfl | SPEAKER_Tfr | SPEAKER_Trl | SPEAKER_Trr;
pub const SPARR_50_5 : SpeakerArrangement = 100;

/** L R C Lfe Ls Rs Tc Tfl Tfr Trl Trr */				// 5.1.5
pub const SPARR_101 : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs  | 
                                           SPEAKER_Tc | SPEAKER_Tfl | SPEAKER_Tfr | SPEAKER_Trl | SPEAKER_Trr;
pub const SPARR_101MPEG3D : SpeakerArrangement = 101;
pub const SPARR_51_5 : SpeakerArrangement = 101;

/** L R C Lfe Ls Rs Tfl Tfc Tfr Trl Trr Lfe2 */			// 5.2.5
pub const SPARR_102 : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C  | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs  |
                                           SPEAKER_Tfl| SPEAKER_Tfc | SPEAKER_Tfr | SPEAKER_Trl | SPEAKER_Trr | SPEAKER_Lfe2;
pub const SPARR_52_5 : SpeakerArrangement = 102;

/** L R C Ls Rs Tc Tfl Tfc Tfr Trl Trr */				// 5.0.6
pub const SPARR_110 : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C | SPEAKER_Ls  | SPEAKER_Rs |
                                           SPEAKER_Tc | SPEAKER_Tfl | SPEAKER_Tfc | SPEAKER_Tfr | SPEAKER_Trl | SPEAKER_Trr;
pub const SPARR_50_6 : SpeakerArrangement = 110;

/** L R C Lfe Ls Rs Tc Tfl Tfc Tfr Trl Trr */			// 5.1.6
pub const SPARR_111 : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs | 
                                           SPEAKER_Tc | SPEAKER_Tfl | SPEAKER_Tfc | SPEAKER_Tfr | SPEAKER_Trl | SPEAKER_Trr;
pub const SPARR_51_6 : SpeakerArrangement = 111;

/** L R C Lfe Ls Rs Lc Rc Tfl Tfc Tfr Trl Trr Lfe2 */	// 7.2.5
pub const SPARR_122 : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C  | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs	| SPEAKER_Lc  | SPEAKER_Rc |
                                           SPEAKER_Tfl| SPEAKER_Tfc | SPEAKER_Tfr | SPEAKER_Trl | SPEAKER_Trr | SPEAKER_Lfe2;
pub const SPARR_72_5 : SpeakerArrangement = 122;

/** L R C Ls Rs Sl Sr Tc Tfl Tfc Tfr Trl Trr */			// 7.0.6
pub const SPARR_130 : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C | SPEAKER_Ls  | SPEAKER_Rs | SPEAKER_Sl | SPEAKER_Sr |
                                           SPEAKER_Tc | SPEAKER_Tfl | SPEAKER_Tfc | SPEAKER_Tfr | SPEAKER_Trl | SPEAKER_Trr;

/** L R C Lfe Ls Rs Sl Sr Tc Tfl Tfc Tfr Trl Trr */		// 7.1.6
pub const SPARR_131 : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Sl | SPEAKER_Sr |
                                           SPEAKER_Tc | SPEAKER_Tfl | SPEAKER_Tfc | SPEAKER_Tfr | SPEAKER_Trl | SPEAKER_Trr;

/** L R Ls Rs Sl Sr Tfl Tfr Trl Trr Bfl Bfr Brl Brr */	// 6.0.4.4
pub const SPARR_140 : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Sl | SPEAKER_Sr |
                                           SPEAKER_Tfl | SPEAKER_Tfr | SPEAKER_Trl | SPEAKER_Trr |
                                           SPEAKER_Bfl | SPEAKER_Bfr | SPEAKER_Brl | SPEAKER_Brr;
pub const SPARR_60_4_4 : SpeakerArrangement = 140;

/** L R C Ls Rs Lc Rc Cs Sl Sr Tc Tfl Tfc Tfr Trl Trc Trr Tsl Tsr Bfl Bfc Bfr */			// 10.0.9.3 (ITU 9+10+3.0 Sound System H)
pub const SPARR_220 : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C  | SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Lc | SPEAKER_Rc | SPEAKER_Cs | SPEAKER_Sl | SPEAKER_Sr | 
                                           SPEAKER_Tc | SPEAKER_Tfl | SPEAKER_Tfc | SPEAKER_Tfr | SPEAKER_Trl | SPEAKER_Trc | SPEAKER_Trr | SPEAKER_Tsl | SPEAKER_Tsr | 
                                           SPEAKER_Bfl| SPEAKER_Bfc | SPEAKER_Bfr;
pub const SPARR_100_9_3 : SpeakerArrangement = 220;

/** L R C Lfe Ls Rs Lc Rc Cs Sl Sr Tc Tfl Tfc Tfr Trl Trc Trr Lfe2 Tsl Tsr Bfl Bfc Bfr */	// 10.2.9.3 (ITU 9+10+3.2 Sound System H)
pub const SPARR_222 : SpeakerArrangement = SPEAKER_L  | SPEAKER_R | SPEAKER_C  | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Lc | SPEAKER_Rc | SPEAKER_Cs | SPEAKER_Sl | SPEAKER_Sr | 
                                           SPEAKER_Tc | SPEAKER_Tfl | SPEAKER_Tfc | SPEAKER_Tfr | SPEAKER_Trl | SPEAKER_Trc | SPEAKER_Trr | SPEAKER_Lfe2 | SPEAKER_Tsl | SPEAKER_Tsr | 
                                           SPEAKER_Bfl| SPEAKER_Bfc | SPEAKER_Bfr;
pub const SPARR_102_9_3 : SpeakerArrangement = 222;

/** L R C Ls Rs Tfl Tfc Tfr Trl Trr Bfl Bfc Bfr */		// 5.0.5.3
pub const SPARR_50_5_3 : SpeakerArrangement = SPEAKER_L | SPEAKER_R | SPEAKER_C | SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Tfl | SPEAKER_Tfc | SPEAKER_Tfr | SPEAKER_Trl | SPEAKER_Trr |
								   SPEAKER_Bfl | SPEAKER_Bfc | SPEAKER_Bfr;

/** L R C Lfe Ls Rs Tfl Tfc Tfr Trl Trr Bfl Bfc Bfr */	// 5.1.5.3
pub const SPARR_51_5_3 : SpeakerArrangement = SPEAKER_L | SPEAKER_R | SPEAKER_C | SPEAKER_Lfe | SPEAKER_Ls | SPEAKER_Rs | SPEAKER_Tfl | SPEAKER_Tfc | SPEAKER_Tfr | SPEAKER_Trl | SPEAKER_Trr | 
								   SPEAKER_Bfl | SPEAKER_Bfc | SPEAKER_Bfr;

