use super::*;
use rand::prelude::*;

pub struct GenAi;

impl AiPlayer<UltimateGame> for GenAi {
    fn make_move(&self, game: &UltimateGame, player_id: usize) -> usize {
        let round_rand = 0.0;
        let mut rng = thread_rng();

        let mut inputs = Vec::new();
        inputs.clear();
        inputs.push(round_rand);

        for field in game.field.iter() {
            for entry in field.iter() {
                let val = match entry {
                    None => 0.0,
                    Some(i) => 2.0 * ((*i == player_id) as usize as f64) - 1.0,
                };
                inputs.push(val);
            }
        }

        for i in 0..9 {
            let cell = (i == game.cell) as usize as f64;
            let finished = game.field[i].status().is_finished() as usize as f64;
            let won = match game.field[i].winner() {
                None => 0.0,
                Some(i) => 2.0 * ((i == player_id) as usize as f64) - 1.0,
            };
            inputs.push(cell);
            inputs.push(finished);
            inputs.push(won);
            inputs.push(rng.gen());
        }

        let mut rating: Vec<_> = rate_moves(&inputs)
            .into_iter()
            .map(|v| if v.is_nan() { 0.0 } else { v })
            .enumerate()
            .collect();

        rating.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        let mut m_game = game.clone();
        for m in 0..9 {
            let status = m_game.make_move(m, player_id);
            if status != GameState::Invalid {
                return m;
            }
        }

        0
    }
}

fn rate_moves(input: &[f64]) -> Vec<f64> {
    // AUTO-GENERATED CODE //

    // Input in 0..118
    let mut v = vec![0.0_f64; 551];

    for i in 0..118 {
        v[i] = input[i];
    }

    v[118] = if v[25] < 0.0 { v[25] * 0.1 } else { v[25] };
    v[119] = v[118].max(0.0).min(1.0);
    v[120] = v[119];
    v[121] = v[120];
    v[122] = v[121].powf(0.5);
    v[123] = v[122].max(0.0).min(1.0);
    v[124] = 1.0;
    v[125] = v[124].abs();
    v[126] = 0.0;
    v[127] = v[126].min(0.0);
    v[128] = v[127] + v[120];
    v[129] = v[128].min(0.0);
    v[130] = if v[129] < 0.0 { v[129] * 0.1 } else { v[129] };
    v[131] = v[130];
    v[132] = if v[95] > 0.0 { v[96] } else { v[131] };
    v[133] = if v[117] < v[132] { 1.0 } else { -1.0 };
    v[134] = v[133].powf(0.5);
    v[135] = if v[125] > 0.0 { v[134] } else { v[64] };
    v[136] = 1.0;
    v[137] = v[136].max(0.0).min(1.0);
    v[138] = v[137] * v[14];
    v[139] = -v[138];
    v[140] = v[139].max(v[97]).min(v[61]);
    v[141] = 0.0;
    v[142] = 1.0;
    v[143] = v[142].powf(0.5);
    v[144] = if v[70] > 0.0 { v[141] } else { v[143] };
    v[145] = if v[144] < 0.0 { v[144] * 0.1 } else { v[144] };
    v[146] = 1.0;
    v[147] = v[146].powf(0.5);
    v[148] = if v[145] > 0.0 { v[147] } else { v[45] };
    v[149] = v[148].abs();
    v[150] = if v[140] > 0.0 { v[38] } else { v[149] };
    v[151] = v[10].min(0.0);
    v[152] = v[150].max(v[84]).min(v[151]);
    v[153] = v[135] + v[152];
    v[154] = v[153].max(0.0);
    v[155] = if v[151] < 0.0 { v[151] * 0.1 } else { v[151] };
    v[156] = 1.0;
    v[157] = v[155] - v[156];
    v[158] = v[157] - v[27];
    v[159] = v[87].max(0.0);
    v[160] = 0.0;
    v[161] = v[9].max(v[159]).min(v[160]);
    v[162] = v[12].min(0.0);
    v[163] = v[161].max(v[162]).min(v[80]);
    v[164] = if v[154] > 0.0 { v[158] } else { v[163] };
    v[165] = v[164].max(0.0);
    v[166] = v[165].powf(0.5);
    v[167] = v[3] * v[166];
    v[168] = 1.0;
    v[169] = v[44] * v[168];
    v[170] = if v[169] < 0.0 { v[169] * 0.1 } else { v[169] };
    v[171] = if v[87] < 0.0 { v[87] * 0.1 } else { v[87] };
    v[172] = if v[167] > 0.0 { v[170] } else { v[171] };
    v[173] = v[172].powf(0.5);
    v[174] = 0.0;
    v[175] = 1.0;
    v[176] = v[114].powf(0.5);
    v[177] = -v[176];
    v[178] = if v[175] < v[177] { 1.0 } else { -1.0 };
    v[179] = v[173].max(v[174]).min(v[178]);
    v[180] = 1.0;
    v[181] = 0.0;
    v[182] = v[181].powf(0.5);
    v[183] = v[182];
    v[184] = v[82].powf(0.5);
    v[185] = v[183].powf(v[184]);
    v[186] = 0.0;
    v[187] = v[186].min(0.0);
    v[188] = v[187].max(0.0).min(1.0);
    v[189] = v[71] * v[188];
    v[190] = if v[185] > 0.0 { v[189] } else { v[175] };
    v[191] = v[58].max(0.0).min(1.0);
    v[192] = if v[191] < v[112] { 1.0 } else { -1.0 };
    v[193] = -v[192];
    v[194] = v[193].min(0.0);
    v[195] = v[194].powf(0.5);
    v[196] = v[195];
    v[197] = if v[196] < 0.0 { v[196] * 0.1 } else { v[196] };
    v[198] = v[197].powf(0.5);
    v[199] = 0.0;
    v[200] = if v[199] < v[151] { 1.0 } else { -1.0 };
    v[201] = v[200].abs();
    v[202] = v[198].max(v[201]).min(v[192]);
    v[203] = if v[202] < 0.0 { v[202] * 0.1 } else { v[202] };
    v[204] = v[190].max(v[203]).min(v[38]);
    v[205] = if v[180] < v[204] { 1.0 } else { -1.0 };
    v[206] = v[205].powf(0.5);
    v[207] = v[206] + v[11];
    v[208] = if v[165] < v[207] { 1.0 } else { -1.0 };
    v[209] = -v[87];
    v[210] = v[209];
    v[211] = if v[208] > 0.0 { v[25] } else { v[210] };
    v[212] = 0.0;
    v[213] = v[77].max(v[211]).min(v[212]);
    v[214] = v[213].powf(v[2]);
    v[215] = -v[86];
    v[216] = if v[156] > 0.0 { v[214] } else { v[215] };
    v[217] = v[216] * v[107];
    v[218] = if v[217] < 0.0 { v[217] * 0.1 } else { v[217] };
    v[219] = v[107].max(v[35]).min(v[94]);
    v[220] = v[218] - v[219];
    v[221] = v[191].max(0.0).min(1.0);
    v[222] = v[221].min(0.0);
    v[223] = v[222] / v[191];
    v[224] = v[223].powf(0.5);
    v[225] = v[224].powf(0.5);
    v[226] = 0.0;
    v[227] = v[226].max(0.0);
    v[228] = v[227].powf(0.5);
    v[229] = 0.0;
    v[230] = if v[228] > 0.0 { v[229] } else { v[39] };
    v[231] = v[225].powf(v[230]);
    v[232] = v[231].min(0.0);
    v[233] = v[232].max(0.0);
    v[234] = -v[233];
    v[235] = 1.0;
    v[236] = v[222];
    v[237] = v[236].abs();
    v[238] = v[235].powf(v[237]);
    v[239] = v[234] * v[238];
    v[240] = 1.0;
    v[241] = if v[219] < v[240] { 1.0 } else { -1.0 };
    v[242] = v[241].min(0.0);
    v[243] = v[16].max(v[242]).min(v[190]);
    v[244] = if v[30] < 0.0 { v[30] * 0.1 } else { v[30] };
    v[245] = v[244].abs();
    v[246] = 1.0;
    v[247] = v[152];
    v[248] = 1.0;
    v[249] = v[247] * v[248];
    v[250] = if v[246] > 0.0 { v[249] } else { v[167] };
    v[251] = if v[245] > 0.0 { v[219] } else { v[250] };
    v[252] = v[71].max(v[243]).min(v[251]);
    v[253] = v[252] * v[55];
    v[254] = v[253] - v[5];
    v[255] = v[254] + v[148];
    v[256] = v[255];
    v[257] = if v[239] > 0.0 { v[141] } else { v[256] };
    v[258] = v[220].max(v[257]).min(v[108]);
    v[259] = v[104] / v[258];
    v[260] = v[238].max(0.0);
    v[261] = v[260].abs();
    v[262] = v[47].max(0.0).min(1.0);
    v[263] = v[262].powf(0.5);
    v[264] = if v[259] > 0.0 { v[261] } else { v[263] };
    v[265] = v[179] * v[264];
    v[266] = 0.0;
    v[267] = v[266].max(0.0);
    v[268] = v[265].powf(v[267]);
    v[269] = v[268].max(0.0).min(1.0);
    v[270] = if v[269] < 0.0 { v[269] * 0.1 } else { v[269] };
    v[271] = v[123] * v[270];
    v[272] = 0.0;
    v[273] = v[272].min(0.0);
    v[274] = v[273].max(0.0);
    v[275] = v[103] / v[271];
    v[276] = v[275].min(0.0);
    v[277] = if v[276] < 0.0 { v[276] * 0.1 } else { v[276] };
    v[278] = v[274].max(v[277]).min(v[55]);
    v[279] = v[151].min(0.0);
    v[280] = v[279] / v[36];
    v[281] = v[73].powf(v[280]);
    v[282] = 1.0;
    v[283] = if v[282] > 0.0 { v[46] } else { v[263] };
    v[284] = 1.0;
    v[285] = v[284];
    v[286] = v[283] / v[285];
    v[287] = 1.0;
    v[288] = if v[281] > 0.0 { v[286] } else { v[287] };
    v[289] = if v[80] < v[288] { 1.0 } else { -1.0 };
    v[290] = 1.0;
    v[291] = -v[290];
    v[292] = v[291];
    v[293] = v[186];
    v[294] = if v[292] > 0.0 { v[293] } else { v[26] };
    v[295] = v[294].max(v[148]).min(v[174]);
    v[296] = 0.0;
    v[297] = v[296].max(0.0).min(1.0);
    v[298] = -v[297];
    v[299] = v[142].abs();
    v[300] = if v[299] < 0.0 { v[299] * 0.1 } else { v[299] };
    v[301] = -v[300];
    v[302] = 0.0;
    v[303] = v[302].powf(0.5);
    v[304] = v[303].max(v[60]).min(v[175]);
    v[305] = v[298].max(v[301]).min(v[304]);
    v[306] = 1.0;
    v[307] = v[306];
    v[308] = v[305] + v[307];
    v[309] = v[235].max(0.0).min(1.0);
    v[310] = if v[309] < 0.0 { v[309] * 0.1 } else { v[309] };
    v[311] = v[213];
    v[312] = v[275].powf(0.5);
    v[313] = -v[312];
    v[314] = v[310].max(v[311]).min(v[313]);
    v[315] = if v[308] > 0.0 { v[142] } else { v[314] };
    v[316] = v[295].max(v[315]).min(v[43]);
    v[317] = v[278].max(v[289]).min(v[316]);
    v[318] = v[213].min(0.0);
    v[319] = 1.0;
    v[320] = v[33] / v[319];
    v[321] = v[105].powf(0.5);
    v[322] = v[320] / v[321];
    v[323] = v[317].max(v[318]).min(v[322]);
    v[324] = v[321].max(0.0).min(1.0);
    v[325] = v[170].max(0.0).min(1.0);
    v[326] = 1.0;
    v[327] = v[326] * v[320];
    v[328] = v[324].max(v[325]).min(v[327]);
    v[329] = v[326].max(v[184]).min(v[7]);
    v[330] = if v[329] > 0.0 { v[278] } else { v[160] };
    v[331] = v[323].max(v[328]).min(v[330]);
    v[332] = if v[271] > 0.0 { v[331] } else { v[287] };
    v[333] = v[24].powf(0.5);
    v[334] = 1.0;
    v[335] = -v[334];
    v[336] = if v[335] < v[50] { 1.0 } else { -1.0 };
    v[337] = v[336].powf(0.5);
    v[338] = -v[170];
    v[339] = if v[338] < 0.0 { v[338] * 0.1 } else { v[338] };
    v[340] = v[297] - v[339];
    v[341] = v[210].min(0.0);
    v[342] = v[341];
    v[343] = if v[340] > 0.0 { v[142] } else { v[342] };
    v[344] = v[343];
    v[345] = if v[38] > 0.0 { v[310] } else { v[344] };
    v[346] = v[324].max(v[337]).min(v[345]);
    v[347] = v[346].max(0.0);
    v[348] = v[115].max(v[333]).min(v[347]);
    v[349] = if v[332] > 0.0 { v[348] } else { v[90] };
    v[350] = if v[349] < v[104] { 1.0 } else { -1.0 };
    v[351] = if v[350] < 0.0 { v[350] * 0.1 } else { v[350] };
    v[352] = 0.0;
    v[353] = v[352].powf(0.5);
    v[354] = if v[351] > 0.0 { v[353] } else { v[168] };
    v[355] = v[39];
    v[356] = v[304].powf(v[200]);
    v[357] = v[310].min(0.0);
    v[358] = v[295].max(0.0);
    v[359] = -v[358];
    v[360] = v[357].powf(v[359]);
    v[361] = v[356] / v[360];
    v[362] = if v[360] > 0.0 { v[116] } else { v[239] };
    v[363] = v[355].max(v[361]).min(v[362]);
    v[364] = v[354] + v[363];
    v[365] = 0.0;
    v[366] = v[342];
    v[367] = if v[365] > 0.0 { v[200] } else { v[366] };
    v[368] = 0.0;
    v[369] = -v[346];
    v[370] = v[219].max(v[369]).min(v[222]);
    v[371] = v[370].max(v[317]).min(v[194]);
    v[372] = v[371] / v[330];
    v[373] = 1.0;
    v[374] = if v[373] < 0.0 { v[373] * 0.1 } else { v[373] };
    v[375] = v[74] - v[284];
    v[376] = v[375] / v[329];
    v[377] = v[376].max(0.0);
    v[378] = -v[377];
    v[379] = v[378].abs();
    v[380] = v[80].max(v[87]).min(v[204]);
    v[381] = 1.0;
    v[382] = v[377].abs();
    v[383] = if v[380] > 0.0 { v[381] } else { v[382] };
    v[384] = v[383].powf(0.5);
    v[385] = v[304].abs();
    v[386] = if v[379] > 0.0 { v[384] } else { v[385] };
    v[387] = v[386].max(0.0).min(1.0);
    v[388] = v[387];
    v[389] = v[374] - v[388];
    v[390] = v[372] * v[389];
    v[391] = v[390].abs();
    v[392] = if v[391] < 0.0 { v[391] * 0.1 } else { v[391] };
    v[393] = v[205].min(0.0);
    v[394] = v[93] - v[393];
    v[395] = v[109].max(0.0);
    v[396] = 0.0;
    v[397] = v[389] + v[396];
    v[398] = v[105] / v[385];
    v[399] = v[249].max(v[398]).min(v[88]);
    v[400] = 0.0;
    v[401] = v[399].max(v[400]).min(v[104]);
    v[402] = if v[286] < 0.0 { v[286] * 0.1 } else { v[286] };
    v[403] = v[402].powf(0.5);
    v[404] = v[332].max(v[403]).min(v[319]);
    v[405] = if v[180] > 0.0 { v[404] } else { v[67] };
    v[406] = v[405].powf(0.5);
    v[407] = v[401].powf(v[406]);
    v[408] = v[407].min(0.0);
    v[409] = v[408].max(0.0);
    v[410] = v[409].abs();
    v[411] = v[410] + v[177];
    v[412] = v[226].max(v[397]).min(v[411]);
    v[413] = v[16].powf(0.5);
    v[414] = 0.0;
    v[415] = v[414];
    v[416] = v[222].abs();
    v[417] = v[41] * v[406];
    v[418] = v[417].max(0.0).min(1.0);
    v[419] = v[418] + v[209];
    v[420] = v[45].max(0.0).min(1.0);
    v[421] = v[148].max(v[420]).min(v[56]);
    v[422] = v[419].max(v[421]).min(v[41]);
    v[423] = v[322].powf(v[15]);
    v[424] = v[422] / v[423];
    v[425] = v[424].min(0.0);
    v[426] = v[415].max(v[416]).min(v[425]);
    v[427] = 1.0;
    v[428] = if v[413] > 0.0 { v[426] } else { v[427] };
    v[429] = v[178].powf(0.5);
    v[430] = v[428].powf(v[429]);
    v[431] = v[430] * v[141];
    v[432] = v[431].abs();
    v[433] = 1.0;
    v[434] = v[433].min(0.0);
    v[435] = if v[434] < v[389] { 1.0 } else { -1.0 };
    v[436] = 0.0;
    v[437] = v[436].max(0.0);
    v[438] = v[437] * v[148];
    v[439] = v[438].powf(0.5);
    v[440] = -v[439];
    v[441] = if v[307] > 0.0 { v[435] } else { v[440] };
    v[442] = v[70].max(v[432]).min(v[441]);
    v[443] = if v[442] < 0.0 { v[442] * 0.1 } else { v[442] };
    v[444] = if v[443] > 0.0 { v[265] } else { v[220] };
    v[445] = if v[396] < v[444] { 1.0 } else { -1.0 };
    v[446] = if v[412] > 0.0 { v[445] } else { v[412] };
    v[447] = 1.0;
    v[448] = -v[447];
    v[449] = v[446] / v[448];
    v[450] = v[449].max(0.0).min(1.0);
    v[451] = v[450].max(0.0);
    v[452] = v[152] / v[451];
    v[453] = if v[395] > 0.0 { v[452] } else { v[122] };
    v[454] = v[394] / v[453];
    v[455] = if v[454] < 0.0 { v[454] * 0.1 } else { v[454] };
    v[456] = if v[455] < v[95] { 1.0 } else { -1.0 };
    v[457] = v[392] / v[456];
    v[458] = if v[368] < v[457] { 1.0 } else { -1.0 };
    v[459] = v[158].abs();
    v[460] = v[168].powf(0.5);
    v[461] = v[460] + v[381];
    v[462] = v[68].max(0.0).min(1.0);
    v[463] = v[462].powf(v[309]);
    v[464] = if v[461] > 0.0 { v[463] } else { v[136] };
    v[465] = if v[93] > 0.0 { v[350] } else { v[464] };
    v[466] = 0.0;
    v[467] = 1.0;
    v[468] = -v[467];
    v[469] = v[466].max(v[352]).min(v[468]);
    v[470] = if v[20] < v[469] { 1.0 } else { -1.0 };
    v[471] = if v[470] > 0.0 { v[327] } else { v[381] };
    v[472] = v[465].max(v[471]).min(v[84]);
    v[473] = -v[472];
    v[474] = -v[473];
    v[475] = v[474].max(0.0);
    v[476] = v[475].min(0.0);
    v[477] = -v[476];
    v[478] = v[459] + v[477];
    v[479] = v[31].abs();
    v[480] = v[199].max(v[371]).min(v[479]);
    v[481] = if v[478] > 0.0 { v[407] } else { v[480] };
    v[482] = 0.0;
    v[483] = v[482].powf(0.5);
    v[484] = v[2].max(v[15]).min(v[459]);
    v[485] = v[14].min(0.0);
    v[486] = v[484].powf(v[485]);
    v[487] = if v[212] > 0.0 { v[332] } else { v[486] };
    v[488] = if v[230] > 0.0 { v[483] } else { v[487] };
    v[489] = if v[16] > 0.0 { v[349] } else { v[114] };
    v[490] = 0.0;
    v[491] = v[53] / v[490];
    v[492] = v[132].max(0.0);
    v[493] = v[492].abs();
    v[494] = v[491] / v[493];
    v[495] = if v[136] > 0.0 { v[263] } else { v[56] };
    v[496] = v[495].max(0.0).min(1.0);
    v[497] = v[496].min(0.0);
    v[498] = if v[489] > 0.0 { v[494] } else { v[497] };
    v[499] = v[498].abs();
    v[500] = v[499].max(0.0);
    v[501] = v[2].powf(v[86]);
    v[502] = v[501];
    v[503] = 1.0;
    v[504] = v[406] + v[142];
    v[505] = v[504].max(0.0);
    v[506] = v[505];
    v[507] = v[502].max(v[503]).min(v[506]);
    v[508] = v[507].max(v[453]).min(v[493]);
    v[509] = if v[508] < v[51] { 1.0 } else { -1.0 };
    v[510] = v[509] * v[244];
    v[511] = if v[488] > 0.0 { v[500] } else { v[510] };
    v[512] = v[175].max(v[83]).min(v[33]);
    v[513] = v[512] * v[348];
    v[514] = v[68] * v[513];
    v[515] = v[514] / v[263];
    v[516] = if v[515] > 0.0 { v[315] } else { v[202] };
    v[517] = v[516].abs();
    v[518] = 0.0;
    v[519] = v[518].powf(v[215]);
    v[520] = 1.0;
    v[521] = if v[519] < v[520] { 1.0 } else { -1.0 };
    v[522] = 0.0;
    v[523] = v[521] * v[522];
    v[524] = v[523] / v[93];
    v[525] = v[517].max(v[524]).min(v[344]);
    v[526] = -v[525];
    v[527] = if v[449] < 0.0 { v[449] * 0.1 } else { v[449] };
    v[528] = v[511].max(v[526]).min(v[527]);
    v[529] = v[481] * v[528];
    v[530] = v[373].max(0.0);
    v[531] = v[530].powf(v[362]);
    v[532] = v[235] + v[478];
    v[533] = v[285] / v[246];
    v[534] = v[533];
    v[535] = v[395].max(v[534]).min(v[429]);
    v[536] = v[532].max(v[535]).min(v[84]);
    v[537] = if v[503] < 0.0 { v[503] * 0.1 } else { v[503] };
    v[538] = v[462].max(v[537]).min(v[430]);
    v[539] = v[439].powf(0.5);
    v[540] = v[436].max(v[538]).min(v[539]);
    v[541] = v[489].min(0.0);
    v[542] = v[316].powf(0.5);
    v[543] = v[451].powf(0.5);
    v[544] = v[543].abs();
    v[545] = v[544];
    v[546] = -v[545];
    v[547] = v[546];
    v[548] = if v[547] < 0.0 { v[547] * 0.1 } else { v[547] };
    v[549] = -v[548];
    v[550] = if v[542] > 0.0 { v[431] } else { v[549] };
    let mut o = vec![0.0; 9];
    o[0] = v[364];
    o[1] = v[367];
    o[2] = v[458];
    o[3] = v[529];
    o[4] = v[531];
    o[5] = v[536];
    o[6] = v[540];
    o[7] = v[541];
    o[8] = v[550];
    o
}
