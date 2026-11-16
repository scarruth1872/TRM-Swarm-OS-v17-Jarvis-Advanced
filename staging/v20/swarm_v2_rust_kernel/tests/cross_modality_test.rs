use std::fs;
use swarm_rust_kernel::scientific_parsers::*;

#[test]
fn test_cross_modality_validation() {
    // 1. Target Identification & Transcriptomics: Sanger (.ab1) + GEO Microarray (.soft)
    let ab1_bytes = fs::read("test_data/3730.ab1").expect("3730.ab1");
    let trace = Ab1Parser::parse(&ab1_bytes).expect("ab1 parse");
    assert_eq!(trace.dye_order, "GATC");
    assert_eq!(trace.base_calls.len(), 1165);

    let soft_text = fs::read_to_string("test_data/soft_ex_affy.soft").expect("soft_ex_affy.soft");
    let geo_samples = GeoSoftParser::parse(&soft_text).expect("geo soft parse");
    assert_eq!(geo_samples.len(), 3);
    assert_eq!(geo_samples[0].platform_id, "GPL72");

    // 2. Structural Biology & Spectroscopy: mmCIF (.cif) + JCAMP-DX (.jdx) + mzML (.mzML)
    let cif_text = fs::read_to_string("test_data/1CRN.cif").expect("1CRN.cif");
    let cif_blocks = CifParser::parse(&cif_text).expect("cif parse");
    let atoms = CifParser::extract_atom_sites(&cif_blocks[0]);
    assert_eq!(atoms.len(), 327);

    let jcamp_text = fs::read_to_string("test_data/acetone_ir.jdx").expect("acetone_ir.jdx");
    let jcamp = JcampParser::parse(&jcamp_text).expect("jcamp parse");
    assert_eq!(jcamp.points.len(), 880);

    let mzml_text = fs::read_to_string("test_data/sample.mzML").expect("sample.mzML");
    let spectra = MzmlParser::parse(&mzml_text).expect("mzml parse");
    assert_eq!(spectra.len(), 2);
    assert_eq!(spectra[0].peaks.len(), 66);

    // 3. Single-Cell & Pharmacokinetics: FCS 3.0 (.fcs) + NONMEM (.csv)
    let fcs_bytes = fs::read("test_data/CFP_Well_A4.fcs").expect("CFP_Well_A4.fcs");
    let fcs = FcsParser::parse(&fcs_bytes).expect("fcs parse");
    assert_eq!(fcs.total_events, 10000);
    assert_eq!(fcs.num_parameters, 16);

    let pk_text = fs::read_to_string("test_data/Theoph.csv").expect("Theoph.csv");
    let pk_subs = PkParser::parse_csv(&pk_text).expect("pk parse");
    assert_eq!(pk_subs.len(), 12);
    let nca = pk_subs[0].compute_nca();
    assert_eq!(nca.c_max, 10.5);

    // 4. Radiology & Clinical Records: DICOM (.dcm) + FHIR (.json)
    let dcm_bytes = fs::read("test_data/CT_small.dcm").expect("CT_small.dcm");
    let dcm = DicomParser::parse(&dcm_bytes).expect("dicom parse");
    assert_eq!(dcm.modality, "CT");
    assert_eq!(dcm.columns, 128);

    let fhir_text = fs::read_to_string("test_data/observation_bp.json").expect("observation_bp.json");
    let obs = FhirParser::parse_observation(&fhir_text).expect("fhir parse");
    assert_eq!(obs.status, "final");

    // 5. Electrophysiology: WFDB ECG (.hea/.dat) + EDF EEG (.edf)
    let hea_text = fs::read_to_string("test_data/100.hea").expect("100.hea");
    let dat_bytes = fs::read("test_data/100.dat").expect("100.dat");
    let ecg_hdr = EcgParser::parse_header(&hea_text).expect("ecg header");
    let ecg_rec = EcgParser::parse_format_212(&dat_bytes, &ecg_hdr).expect("ecg parse");
    let (bpm, r_peaks) = EcgParser::calculate_heart_rate(&ecg_rec.lead_voltages_mv[0], ecg_hdr.sampling_rate_hz);
    assert!(bpm > 65.0 && bpm < 75.0);
    assert_eq!(r_peaks.len(), 24);

    let edf_bytes = fs::read("test_data/sample_eeg.edf").expect("sample_eeg.edf");
    let edf = EdfParser::parse(&edf_bytes).expect("edf parse");
    assert_eq!(edf.header.num_signals, 23);
    assert!(edf.records_parsed >= 10);
    let eeg_stats = edf.channel_stats(0).expect("ch1 stats");
    assert!(eeg_stats.3 > 50.0); // RMS amplitude > 50 uV

    // 6. Cryo-EM 3D Density Volumes: MRC2014 (.map/.mrc)
    let mrc_bytes = fs::read("test_data/emd_3001.map").expect("emd_3001.map");
    let mrc = MrcParser::parse(&mrc_bytes).expect("mrc parse");
    assert_eq!(mrc.header.nx, 73);
    assert_eq!(mrc.header.ny, 43);
    assert_eq!(mrc.header.nz, 25);
    assert_eq!(mrc.data.len(), 73 * 43 * 25);
    let (contour_vox, _mean) = mrc.contour_stats(0.2);
    assert!(contour_vox > 1000);

    // 7. Spatial Transcriptomics: 10x Genomics Visium (.csv)
    let st_text = fs::read_to_string("test_data/tissue_positions_list.csv").expect("tissue_positions_list.csv");
    let st = SpatialParser::parse_visium_csv(&st_text, 55.0).expect("visium parse");
    assert_eq!(st.total_spots, 4992);
    assert!(st.in_tissue_spots > 1000);
    assert_eq!(st.spot_diameter_fullres, 55.0);

    // 8. Surface Plasmon Resonance: Biacore Sensorgram (.csv)
    let spr_text = fs::read_to_string("test_data/biacore_sensorgram.csv").expect("biacore_sensorgram.csv");
    let spr = SprParser::parse(&spr_text, "3P3G_LpxC", "TRM-ABX-220", 50.0).expect("spr parse");
    assert_eq!(spr.points.len(), 226);
    assert!(spr.r_max > 90.0);
    assert!(spr.kd_affinity_nm < 10.0); // Sub-10nM affinity
}


