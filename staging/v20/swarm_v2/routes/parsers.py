# Scientific Parsers FastAPI Router
# TRM Swarm OS v19 - Truth Anchor: 0x4D4F4E4144000001

from fastapi import APIRouter, HTTPException, UploadFile, File, Body, Request
from typing import Dict, Any, Optional
from swarm_v2.parsers.parser_registry import parser_registry

router = APIRouter(prefix="/parsers", tags=["Scientific Parsers"])

@router.get("/status")
def get_parsers_status() -> Dict[str, Any]:
    """Retrieve full inventory of all 20 scientific and system parsers."""
    return parser_registry.get_inventory()

@router.post("/parse")
def parse_payload(
    payload: str = Body(..., media_type="text/plain"),
    format_hint: Optional[str] = None,
    filename: Optional[str] = None
) -> Dict[str, Any]:
    """Parse any incoming scientific payload using auto-detection or format hint."""
    try:
        res = parser_registry.parse(payload, format_hint=format_hint, filename=filename)
        return {"status": "SUCCESS", "result": res}
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))

@router.post("/fastq")
def parse_fastq(payload: str = Body(..., media_type="text/plain")) -> Dict[str, Any]:
    """Targeted parse for FASTQ sequencing reads."""
    try:
        res = parser_registry.parse(payload, format_hint="fastq")
        return {"status": "SUCCESS", "result": res}
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))

@router.post("/vcf")
def parse_vcf(payload: str = Body(..., media_type="text/plain")) -> Dict[str, Any]:
    """Targeted parse for VCF 4.3 genomic variants."""
    try:
        res = parser_registry.parse(payload, format_hint="vcf")
        return {"status": "SUCCESS", "result": res}
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))

@router.post("/fhir")
async def parse_fhir(request: Request) -> Dict[str, Any]:
    """Targeted parse and validation for FHIR R4 resources."""
    try:
        body_bytes = await request.body()
        content_str = body_bytes.decode("utf-8", errors="replace")
        res = parser_registry.parse(content_str, format_hint="fhir")
        return {"status": "SUCCESS", "result": res}
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))

@router.post("/dicom/upload")
async def parse_dicom_file(file: UploadFile = File(...)) -> Dict[str, Any]:
    """Parse binary DICOM image file and extract metadata."""
    try:
        content = await file.read()
        res = parser_registry.parse(content, format_hint="dicom", filename=file.filename)
        return {"status": "SUCCESS", "result": res}
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))

@router.post("/mzml")
async def parse_mzml(request: Request) -> Dict[str, Any]:
    """Targeted parse for mzML mass spectrometry XML."""
    try:
        body_bytes = await request.body()
        content_str = body_bytes.decode("utf-8", errors="replace")
        res = parser_registry.parse(content_str, format_hint="mzml")
        return {"status": "SUCCESS", "result": res}
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))

@router.post("/mmcif")
async def parse_mmcif(request: Request) -> Dict[str, Any]:
    """Targeted parse for mmCIF / PDBx crystallography text."""
    try:
        body_bytes = await request.body()
        content_str = body_bytes.decode("utf-8", errors="replace")
        res = parser_registry.parse(content_str, format_hint="mmcif")
        return {"status": "SUCCESS", "result": res}
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))

@router.post("/netcdf/upload")
async def parse_netcdf_file(file: UploadFile = File(...)) -> Dict[str, Any]:
    """Parse binary NetCDF gridded array dataset."""
    try:
        content = await file.read()
        res = parser_registry.parse(content, format_hint="netcdf", filename=file.filename)
        return {"status": "SUCCESS", "result": res}
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))

@router.post("/abif/upload")
async def parse_abif_file(file: UploadFile = File(...)) -> Dict[str, Any]:
    """Parse binary Sanger ABIF (.ab1) sequencing trace file."""
    try:
        content = await file.read()
        res = parser_registry.parse(content, format_hint="abif", filename=file.filename)
        return {"status": "SUCCESS", "result": res}
    except Exception as e:
        raise HTTPException(status_code=400, detail=str(e))
