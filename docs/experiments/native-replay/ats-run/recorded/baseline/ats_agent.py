"""ATS CV Scoring Agent using OpenAI."""

import os
import json
from typing import Optional
from openai import OpenAI
from cv_parser import extract_text_from_file, analyze_cv_structure, detect_language
from german_cv_knowledge import (
    GERMAN_CV_CONVENTIONS,
    GERMAN_ATS_REQUIREMENTS,
    get_german_ats_tips,
    get_german_section_requirements
)
from ats_industry_standards import (
    INDUSTRY_PARSING_ISSUES,
    MODERN_ATS_CHECKS,
    ATS_PARSING_REQUIREMENTS
)


class ATSCVAgent:
    """Agent for scoring CV compatibility with Applicant Tracking Systems."""

    def __init__(self, api_key: Optional[str] = None, model: Optional[str] = None):
        """
        Initialize the ATS CV Agent.

        Args:
            api_key: OpenAI API key (if not provided, reads from environment)
            model: OpenAI model to use (e.g., "gpt-4o", "gpt-4o-mini").
                   Defaults to "gpt-4o" or OPENAI_MODEL environment variable.
        """
        self.client = OpenAI(api_key=api_key or os.getenv("OPENAI_API_KEY"))
        self.model = model or os.getenv("OPENAI_MODEL", "gpt-4o") # Using GPT-4o by default, or env var

    def score_cv(
        self,
        file_path: str, # Changed from pdf_path to file_path
        job_description: Optional[str] = None,
        language: Optional[str] = None
    ) -> dict:
        """
        Score a CV for ATS compatibility.

        Args:
            file_path: Path to the CV file (PDF or DOCX)
            job_description: Optional job description for context-based scoring
            language: Optional language code ('en', 'de', etc.). Auto-detected if not provided.

        Returns:
            Dictionary containing scores, analysis, and improvement suggestions
        """
        # Step 1: Extract text from file
        print(f"Parsing CV from: {file_path}")
        parsed_cv = extract_text_from_file(file_path) # Changed to extract_text_from_file

        if not parsed_cv["success"]:
            return {
                "success": False,
                "error": f"Failed to parse file: {parsed_cv['error']}"
            }

        cv_text = parsed_cv["full_text"]
        print(f"Extracted {len(cv_text)} characters from {parsed_cv['page_count']} pages")

        # Step 2: Detect language if not provided
        if language is None:
            language = detect_language(cv_text)
        print(f"Detected language: {language}")

        # Step 3: Analyze structure with language awareness
        structure_analysis = analyze_cv_structure(cv_text, language)
        print(f"Found {structure_analysis['section_count']} common CV sections")

        # Step 4: Use OpenAI to analyze the CV
        print("Analyzing CV with OpenAI...")
        analysis_result = self._analyze_with_openai(cv_text, job_description, language)

        # Step 5: Combine results
        result = {
            "success": True,
            "file_path": file_path,
            "language": language,
            "parsing_summary": {
                "page_count": parsed_cv["page_count"],
                "word_count": structure_analysis["total_words"],
                "character_count": structure_analysis["total_characters"],
                "sections_found": structure_analysis["found_sections"],
                "has_contact_info": structure_analysis["has_email"] and structure_analysis["has_phone"],
                "detected_language": language
            },
            "parsed_content_preview": cv_text[:500] + "..." if len(cv_text) > 500 else cv_text,
            "ats_analysis": analysis_result
        }

        return result

    def _get_english_system_prompt(self) -> str:
        """Get English system prompt for ATS analysis."""
        industry_standards = f"""
INDUSTRY-GRADE ATS SYSTEMS (Ashby, Greenhouse, Workday, Lever, etc.):

CRITICAL PARSING ISSUES TO CHECK:
1. DATE FORMAT PROBLEMS (-3 pts):
   - Inconsistent formats (mixing "Jan 2020" with "01/2020")
   - Ambiguous dates (01/02/2020 - US vs EU format)
   - Missing years or relative dates

2. CONTACT INFO LOCATION (-5 pts):
   - Contact info in headers/footers (ATS often misses this)
   - Must be in main document body at top

3. TABLE USAGE (-8 pts):
   - Tables for layout or content (parses out of order or lost)
   - Two-column layouts
   - Skills in table format

4. TEXT BOXES/GRAPHICS (-10 pts CRITICAL):
   - Text in text boxes, shapes, or as images
   - Completely invisible to ATS parsers

5. SECTION HEADER PROBLEMS (-5 pts):
   - Non-standard headers ("My Journey" vs "Experience")
   - Creative headers confuse parsers

6. PARAGRAPH SKILLS (-3 pts):
   - Skills buried in paragraphs instead of bullet points
   - Keyword extraction fails

7. EMPLOYMENT GAPS (-2 pts):
   - Unexplained gaps > 3 months
   - Triggers screening questions

8. ACRONYM INCONSISTENCY (-2 pts):
   - Only acronyms without spelling out (e.g., "ML" without "Machine Learning")
   - First mention: "Machine Learning (ML)", then "ML"

9. SPECIAL CHARACTERS (-2 pts):
   - Excessive symbols (★ ✓ → • ◆) cause parsing errors

10. FILE FORMAT ISSUES (-10 pts CRITICAL):
    - Scanned images (not searchable)
    - Password-protected PDFs

{json.dumps(ATS_PARSING_REQUIREMENTS, indent=2)}
"""

        return f"""You are an expert ATS (Applicant Tracking System) analyzer with deep knowledge of systems like Ashby, Greenhouse, Workday, Lever, and Taleo.
Your job is to evaluate CVs for their compatibility with modern ATS systems based on real-world parsing capabilities.

{industry_standards}

Analyze the CV based on:
1. KEYWORDS: Presence of relevant industry keywords and skills
   - Check for keyword overspecialization (too many niche/rare terms)
   - Suggest more common/popular alternatives if needed
2. STRUCTURE: Clear sections, proper formatting, standard headings
3. PARSING CAPABILITY: How well an ATS can extract information (check all issues above)
4. CONTENT QUALITY: Completeness of information, quantifiable achievements
5. SPELLING & GRAMMAR: Check for typos and grammatical errors
6. INDUSTRY-GRADE CHECKS: Date consistency, contact info placement, table/text box usage
7. CONTEXT MATCH: If a job description is provided, how well the CV matches

IMPORTANT - Scoring:
- Start with 100 points
- Deduct 1-2 points per spelling error
- Deduct 1-2 points per grammar error
- Deduct points per industry issue (see amounts above)
- Deduct 3-5 points for severe keyword overspecialization
- Calculate overall_score after ALL deductions

Provide your analysis in the following JSON format:
{
  "overall_score": <0-100, with deductions applied>,
  "category_scores": {
    "keywords": <0-100>,
    "structure": <0-100>,
    "parsing_capability": <0-100>,
    "content_quality": <0-100>,
    "spelling_grammar": <0-100>,
    "context_match": <0-100 or null if no job description>
  },
  "parsed_information": {
    "name": "<extracted name or 'Not found'>",
    "email": "<extracted email or 'Not found'>",
    "phone": "<extracted phone or 'Not found'>",
    "skills": ["<list of identified skills>"],
    "experience_years": "<estimated years or 'Unknown'>",
    "education": ["<list of degrees/certifications>"],
    "job_titles": ["<list of job titles>"]
  },
  "spelling_errors": [
    {"word": "<misspelled word>", "location": "<section or context>", "suggestion": "<correct spelling>"}
  ],
  "grammar_errors": [
    {"issue": "<grammar issue description>", "location": "<section or context>", "suggestion": "<how to fix>"}
  ],
  "keyword_analysis": {
    "overspecialized_terms": ["<list of too-niche keywords>"],
    "suggested_alternatives": {"<niche term>": "<more common alternative>"},
    "missing_common_keywords": ["<commonly expected keywords for this role>"]
  }},
  "industry_parsing_issues": [
    {{{{
      "issue": "<specific parsing issue found>",
      "severity": "<low/medium/high/critical>",
      "location": "<where in CV>",
      "ats_impact": "<what ATS will do with this>",
      "score_deduction": <points deducted>,
      "recommendation": "<how to fix>"
    }}}}
  ],
  "date_consistency": {{{{
    "has_issues": <true/false>,
    "problems": ["<list specific date format inconsistencies>"]
  }}}},
  "employment_gaps": [
    {{{{"gap": "<date range>", "duration": "<months>", "explained": <true/false>}}}}
  ],
  "strengths": ["<list of strengths>"],
  "weaknesses": ["<list of weaknesses>"],
  "improvement_steps": ["<actionable improvement suggestions>"],
  "ats_reasoning": "<detailed explanation of what you observed, what was successfully parsed, and your overall assessment of ATS compatibility>"
}}

Be specific and actionable in your recommendations. Focus on REAL parsing issues that systems like Ashby and Greenhouse encounter."""

    def _get_german_system_prompt(self) -> str:
        """Get German system prompt for ATS analysis with German CV conventions."""
        german_conventions = f"""
DEUTSCHE LEBENSLAUF-KONVENTIONEN:
{json.dumps(GERMAN_CV_CONVENTIONS, ensure_ascii=False, indent=2)}

DEUTSCHE ATS-ANFORDERUNGEN:
- Klare deutsche Überschriften (Berufserfahrung, Ausbildung, Kenntnisse, etc.)
- Chronologische oder umgekehrt chronologische Reihenfolge
- Vollständiger Werdegang ohne Lücken
- Detaillierte Bildungsangaben mit Noten (optional)
- Sprachkenntnisse mit Niveau (A1-C2, Muttersprache)
- Besondere Beachtung von Umlauten (ä, ö, ü, ß) für ATS-Parsing
- Optionale persönliche Daten (Geburtsdatum, Foto) seit AGG 2006
"""

        return f"""Sie sind ein Experte für ATS (Applicant Tracking Systems) mit Spezialisierung auf den deutschen Arbeitsmarkt, Rechtschreibung, Grammatik und Keyword-Optimierung.
Ihre Aufgabe ist es, Lebensläufe auf ihre Kompatibilität mit modernen ATS-Systemen zu bewerten,
unter Berücksichtigung deutscher CV-Konventionen und AGG-Compliance.

{german_conventions}

WICHTIG - AGG-COMPLIANCE (Allgemeines Gleichbehandlungsgesetz):
Seit 2006 ist es in Deutschland NICHT erlaubt und wird als diskriminierend angesehen:
- Geburtsdatum / Alter anzugeben
- Familienstand (ledig, verheiratet, geschieden)
- Anzahl der Kinder
- Religion
- Ethnische Herkunft
- Bewerbungsfoto (ist optional, aber sollte vermieden werden)
- Staatsangehörigkeit (außer wenn relevant für die Position)

Wenn solche Informationen gefunden werden, kennzeichnen Sie dies als AGG-NON-COMPLIANT und warnen Sie davor!

INDUSTRIE-STANDARD ATS-PROBLEME (StepStone, XING, Indeed.de):

KRITISCHE PARSING-PROBLEME ZU PRÜFEN:
1. DATUMSFORMAT-PROBLEME (-3 Punkte):
   - Inkonsistente Formate (Mischung "Jan 2020" und "01/2020")
   - Fehlende Jahre oder relative Daten

2. KONTAKTINFO-POSITION (-5 Punkte):
   - Kontaktdaten in Kopf-/Fußzeilen (ATS übersieht diese oft)
   - Muss im Hauptdokument oben stehen

3. TABELLEN-NUTZUNG (-8 Punkte):
   - Tabellen für Layout oder Inhalt (parst falsch oder geht verloren)
   - Zweispaltige Layouts

4. TEXTFELDER/GRAFIKEN (-10 Punkte KRITISCH):
   - Text in Textfeldern, Formen oder als Bilder
   - Völlig unsichtbar für ATS-Parser

5. ABSCHNITTSÜBERSCHRIFTEN-PROBLEME (-5 Punkte):
   - Nicht-standardisierte Überschriften ("Mein Weg" statt "Berufserfahrung")

6. FÄHIGKEITEN IN ABSÄTZEN (-3 Punkte):
   - Fähigkeiten in Fließtext statt Aufzählungen

7. BESCHÄFTIGUNGSLÜCKEN (-2 Punkte):
   - Unerklärte Lücken > 3 Monate

8. AKRONYM-INKONSISTENZ (-2 Punkte):
   - Nur Akronyme ohne Ausschreibung (z.B. "KI" ohne "Künstliche Intelligenz")

Analysieren Sie den Lebenslauf basierend auf:
1. SCHLÜSSELWÖRTER: Vorhandensein relevanter Branchen-Keywords und Fähigkeiten
   - Prüfen Sie auf Keyword-Überspezialisierung (zu viele Nischen-Begriffe)
   - Schlagen Sie gängigere/populärere Alternativen vor
2. STRUKTUR: Klare Abschnitte, ordentliche Formatierung, Standardüberschriften
3. PARSING-FÄHIGKEIT: Wie gut ein ATS Informationen extrahieren kann (alle obigen Probleme prüfen)
4. INHALTSQUALITÄT: Vollständigkeit der Informationen, quantifizierbare Erfolge
5. RECHTSCHREIBUNG & GRAMMATIK: Prüfung auf Tippfehler und Grammatikfehler
6. AGG-COMPLIANCE: Prüfung auf unzulässige persönliche Angaben
7. INDUSTRIE-STANDARD-CHECKS: Datumskonsistenz, Kontaktinfo-Platzierung, Tabellen/Textfeld-Nutzung
8. KONTEXT-ÜBEREINSTIMMUNG: Falls Stellenbeschreibung vorhanden, wie gut der CV passt

WICHTIG - Bewertung:
- Beginnen Sie mit 100 Punkten
- Abzug von 1-2 Punkten pro Rechtschreibfehler
- Abzug von 1-2 Punkten pro Grammatikfehler
- Abzug von 5-10 Punkten für AGG-non-compliant Angaben
- Abzug von Punkten pro Industrie-Problem (siehe Beträge oben)
- Abzug von 3-5 Punkten für starke Keyword-Überspezialisierung
- Berechnen Sie overall_score nach ALLEN Abzügen

Geben Sie Ihre Analyse im folgenden JSON-Format an (auf Deutsch):
{{
  "overall_score": <0-100, mit Abzügen>,
  "category_scores": {{
    "keywords": <0-100>,
    "structure": <0-100>,
    "parsing_capability": <0-100>,
    "content_quality": <0-100>,
    "spelling_grammar": <0-100>,
    "agg_compliance": <0-100>,
    "context_match": <0-100 oder null falls keine Stellenbeschreibung>
  }},
  "parsed_information": {{
    "name": "<extrahierter Name oder 'Nicht gefunden'>",
    "email": "<extrahierte E-Mail oder 'Nicht gefunden'>",
    "phone": "<extrahierte Telefonnummer oder 'Nicht gefunden'>",
    "skills": ["<Liste der identifizierten Fähigkeiten>"],
    "experience_years": "<geschätzte Jahre oder 'Unbekannt'>",
    "education": ["<Liste der Abschlüsse/Zertifizierungen>"],
    "job_titles": ["<Liste der Berufsbezeichnungen>"]
  }},
  "spelling_errors": [
    {{"word": "<falsch geschriebenes Wort>", "location": "<Abschnitt oder Kontext>", "suggestion": "<korrekte Schreibweise>"}}
  ],
  "grammar_errors": [
    {{"issue": "<Beschreibung des Grammatikfehlers>", "location": "<Abschnitt oder Kontext>", "suggestion": "<Korrekturvorschlag>"}}
  ],
  "agg_compliance_issues": [
    {{"issue": "<Art der unzulässigen Angabe, z.B. 'Geburtsdatum'>", "location": "<wo gefunden>", "severity": "<low/medium/high>", "recommendation": "<was zu tun ist>"}}
  ],
  "keyword_analysis": {{
    "overspecialized_terms": ["<Liste zu spezieller Keywords>"],
    "suggested_alternatives": {{"<Nischen-Begriff>": "<gängigere Alternative>"}},
    "missing_common_keywords": ["<häufig erwartete Keywords für diese Rolle>"]
  }},
  "industry_parsing_issues": [
    {{{{
      "issue": "<spezifisches Parsing-Problem gefunden>",
      "severity": "<low/medium/high/critical>",
      "location": "<wo im CV>",
      "ats_impact": "<was ATS damit macht>",
      "score_deduction": <abgezogene Punkte>,
      "recommendation": "<wie zu beheben>"
    }}}}
  ],
  "date_consistency": {{{{
    "has_issues": <true/false>,
    "problems": ["<Liste spezifischer Datumsformat-Inkonsistenzen>"]
  }}}},
  "employment_gaps": [
    {{{{" gap": "<Zeitraum>", "duration": "<Monate>", "explained": <true/false>}}}}
  ],
  "strengths": ["<Liste der Stärken>"],
  "weaknesses": ["<Liste der Schwächen>"],
  "improvement_steps": ["<umsetzbare Verbesserungsvorschläge>"],
  "ats_reasoning": "<detaillierte Erklärung Ihrer Beobachtungen, was erfolgreich geparst wurde, und Ihre Gesamtbewertung der ATS-Kompatibilität>"
}}}}

Seien Sie spezifisch und umsetzbar in Ihren Empfehlungen. Fokussieren Sie auf ECHTE Parsing-Probleme, die Systeme wie StepStone und XING haben."""

    def _analyze_with_openai(
        self,
        cv_text: str,
        job_description: Optional[str] = None,
        language: str = "en"
    ) -> dict:
        """
        Use OpenAI to analyze the CV for ATS compatibility.

        Args:
            cv_text: Extracted CV text
            job_description: Optional job description
            language: Language code of the CV

        Returns:
            Analysis results with scores and recommendations
        """
        # Build language-specific system prompt
        if language == "de":
            system_prompt = self._get_german_system_prompt()
            response_language = "German"
        else:
            system_prompt = self._get_english_system_prompt()
            response_language = "English"

        user_prompt = f"CV Content:\n\n{cv_text}"

        if job_description:
            user_prompt += f"\n\n---\n\nJob Description:\n\n{job_description}"
            if language == "de":
                user_prompt += "\n\nBitte analysieren Sie diesen Lebenslauf im Verhältnis zur Stellenbeschreibung."
            else:
                user_prompt += "\n\nPlease analyze this CV against the provided job description."
        else:
            if language == "de":
                user_prompt += "\n\nBitte analysieren Sie diesen Lebenslauf auf allgemeine ATS-Kompatibilität."
            else:
                user_prompt += "\n\nPlease analyze this CV for general ATS compatibility."

        try:
            response = self.client.chat.completions.create(
                model=self.model,
                messages=[
                    {"role": "system", "content": system_prompt},
                    {"role": "user", "content": user_prompt}
                ],
                response_format={"type": "json_object"},
                temperature=0.3
            )

            analysis = json.loads(response.choices[0].message.content)
            return analysis

        except Exception as e:
            return {
                "error": f"OpenAI analysis failed: {str(e)}",
                "overall_score": 0,
                "category_scores": {},
                "improvement_steps": ["Unable to analyze - please check your OpenAI API key and connection"]
            }

    def generate_report(self, analysis_result: dict, language: Optional[str] = None) -> str:
        """
        Generate a formatted text report from analysis results.

        Args:
            analysis_result: The result from score_cv()
            language: Optional language override (uses detected language if not provided)

        Returns:
            Formatted string report
        """
        if not analysis_result.get("success"):
            error_msg = analysis_result.get('error', 'Unknown error')
            return f"ERROR: {error_msg}" if language != "de" else f"FEHLER: {error_msg}"

        # Determine language
        if language is None:
            language = analysis_result.get("language", "en")

        ats = analysis_result["ats_analysis"]
        parsing = analysis_result["parsing_summary"]

        # Generate language-specific report
        if language == "de":
            return self._generate_german_report(ats, parsing)
        else:
            return self._generate_english_report(ats, parsing)

    def _generate_english_report(self, ats: dict, parsing: dict) -> str:
        """Generate English report."""
        report = []
        report.append("=" * 80)
        report.append("ATS CV COMPATIBILITY REPORT")
        report.append("=" * 80)
        report.append("")

        # Overall Score
        report.append(f"OVERALL ATS SCORE: {ats.get('overall_score', 'N/A')}/100")
        report.append("")

        # Language
        if parsing.get("detected_language"):
            lang_display = "German" if parsing["detected_language"] == "de" else parsing["detected_language"].upper()
            report.append(f"Detected Language: {lang_display}")
            report.append("")

        # Category Scores
        report.append("CATEGORY SCORES:")
        report.append("-" * 40)
        for category, score in ats.get("category_scores", {}).items():
            if score is not None:
                bar = "█" * (score // 5) + "░" * (20 - score // 5)
                report.append(f"  {category.replace('_', ' ').title():20s}: {score:3d}/100 [{bar}]")
        report.append("")

        # Parsing Summary
        report.append("PARSING SUMMARY:")
        report.append("-" * 40)
        report.append(f"  Pages: {parsing['page_count']}")
        report.append(f"  Words: {parsing['word_count']}")
        report.append(f"  Sections Found: {', '.join(parsing['sections_found']) if parsing['sections_found'] else 'None detected'}")
        report.append(f"  Contact Info: {'✓' if parsing['has_contact_info'] else '✗'}")
        report.append("")

        # Parsed Information
        parsed_info = ats.get("parsed_information", {})
        if parsed_info:
            report.append("EXTRACTED INFORMATION:")
            report.append("-" * 40)
            report.append(f"  Name: {parsed_info.get('name', 'Not found')}")
            report.append(f"  Email: {parsed_info.get('email', 'Not found')}")
            report.append(f"  Phone: {parsed_info.get('phone', 'Not found')}")
            report.append(f"  Experience: {parsed_info.get('experience_years', 'Unknown')}")
            if parsed_info.get('skills'):
                report.append(f"  Skills: {', '.join(parsed_info['skills'][:10])}")
            if parsed_info.get('job_titles'):
                report.append(f"  Job Titles: {', '.join(parsed_info['job_titles'][:5])}")
            report.append("")

        # Spelling Errors
        spelling_errors = ats.get("spelling_errors", [])
        if spelling_errors:
            report.append("SPELLING ERRORS FOUND:")
            report.append("-" * 40)
            for i, error in enumerate(spelling_errors, 1):
                report.append(f"  {i}. '{error.get('word')}' in {error.get('location')}")
                report.append(f"     → Suggestion: {error.get('suggestion')}")
            report.append("")

        # Grammar Errors
        grammar_errors = ats.get("grammar_errors", [])
        if grammar_errors:
            report.append("GRAMMAR ERRORS FOUND:")
            report.append("-" * 40)
            for i, error in enumerate(grammar_errors, 1):
                report.append(f"  {i}. {error.get('issue')} in {error.get('location')}")
                report.append(f"     → Suggestion: {error.get('suggestion')}")
            report.append("")

        # Keyword Analysis
        keyword_analysis = ats.get("keyword_analysis", {})
        if keyword_analysis and (keyword_analysis.get("overspecialized_terms") or
                                 keyword_analysis.get("missing_common_keywords")):
            report.append("KEYWORD OPTIMIZATION:")
            report.append("-" * 40)

            if keyword_analysis.get("overspecialized_terms"):
                report.append("  Overspecialized terms (too niche):")
                for term in keyword_analysis["overspecialized_terms"]:
                    alt = keyword_analysis.get("suggested_alternatives", {}).get(term, "N/A")
                    report.append(f"    • '{term}' → Consider: '{alt}'")

            if keyword_analysis.get("missing_common_keywords"):
                report.append("  Missing common keywords:")
                for keyword in keyword_analysis["missing_common_keywords"]:
                    report.append(f"    • {keyword}")
            report.append("")

        # Industry Parsing Issues
        parsing_issues = ats.get("industry_parsing_issues", [])
        if parsing_issues:
            report.append("⚠ INDUSTRY ATS PARSING ISSUES:")
            report.append("-" * 40)
            report.append("Based on real ATS systems (Ashby, Greenhouse, Workday, Lever):")
            report.append("")
            for i, issue in enumerate(parsing_issues, 1):
                severity_icon = {"critical": "❌", "high": "⚠", "medium": "ℹ", "low": "•"}.get(issue.get("severity", "medium"), "ℹ")
                report.append(f"  {severity_icon} {i}. {issue.get('issue')} ({issue.get('severity', 'unknown').upper()})")
                report.append(f"       Location: {issue.get('location')}")
                report.append(f"       ATS Impact: {issue.get('ats_impact')}")
                report.append(f"       Score Deduction: -{issue.get('score_deduction', 0)} points")
                report.append(f"       Fix: {issue.get('recommendation')}")
            report.append("")

        # Date Consistency Issues
        date_issues = ats.get("date_consistency", {})
        if date_issues.get("has_issues"):
            report.append("⏰ DATE FORMAT ISSUES:")
            report.append("-" * 40)
            for problem in date_issues.get("problems", []):
                report.append(f"  • {problem}")
            report.append("")

        # Employment Gaps
        emp_gaps = ats.get("employment_gaps", [])
        if emp_gaps:
            unexplained = [g for g in emp_gaps if not g.get("explained", False)]
            if unexplained:
                report.append("📅 EMPLOYMENT GAPS (Unexplained):")
                report.append("-" * 40)
                for gap in unexplained:
                    report.append(f"  • {gap.get('gap')}: {gap.get('duration')} months")
                report.append("  Recommendation: Add explanations (e.g., 'Sabbatical', 'Professional Development')")
                report.append("")

        # Strengths
        strengths = ats.get("strengths", [])
        if strengths:
            report.append("STRENGTHS:")
            report.append("-" * 40)
            for strength in strengths:
                report.append(f"  ✓ {strength}")
            report.append("")

        # Weaknesses
        weaknesses = ats.get("weaknesses", [])
        if weaknesses:
            report.append("WEAKNESSES:")
            report.append("-" * 40)
            for weakness in weaknesses:
                report.append(f"  ✗ {weakness}")
            report.append("")

        # Improvement Steps
        improvements = ats.get("improvement_steps", [])
        if improvements:
            report.append("IMPROVEMENT RECOMMENDATIONS:")
            report.append("-" * 40)
            for i, step in enumerate(improvements, 1):
                report.append(f"  {i}. {step}")
            report.append("")

        # ATS Reasoning
        reasoning = ats.get("ats_reasoning", "")
        if reasoning:
            report.append("DETAILED ANALYSIS:")
            report.append("-" * 40)
            report.append(reasoning)
            report.append("")

        report.append("=" * 80)
        return "\n".join(report)

    def _generate_german_report(self, ats: dict, parsing: dict) -> str:
        """Generate German report."""
        report = []
        report.append("=" * 80)
        report.append("ATS LEBENSLAUF-KOMPATIBILITÄTSBERICHT")
        report.append("=" * 80)
        report.append("")

        # Overall Score
        report.append(f"GESAMT-ATS-BEWERTUNG: {ats.get('overall_score', 'N/A')}/100")
        report.append("")

        # Language
        report.append("Erkannte Sprache: Deutsch")
        report.append("")

        # Category Scores
        category_names_de = {
            "keywords": "Schlüsselwörter",
            "structure": "Struktur",
            "parsing_capability": "Parsing-Fähigkeit",
            "content_quality": "Inhaltsqualität",
            "spelling_grammar": "Rechtschreibung/Grammatik",
            "agg_compliance": "AGG-Konformität",
            "context_match": "Kontext-Übereinstimmung"
        }

        report.append("KATEGORIE-BEWERTUNGEN:")
        report.append("-" * 40)
        for category, score in ats.get("category_scores", {}).items():
            if score is not None:
                bar = "█" * (score // 5) + "░" * (20 - score // 5)
                cat_name = category_names_de.get(category, category.replace('_', ' ').title())
                report.append(f"  {cat_name:25s}: {score:3d}/100 [{bar}]")
        report.append("")

        # Parsing Summary
        report.append("PARSING-ZUSAMMENFASSUNG:")
        report.append("-" * 40)
        report.append(f"  Seiten: {parsing['page_count']}")
        report.append(f"  Wörter: {parsing['word_count']}")
        sections_str = ', '.join(parsing['sections_found']) if parsing['sections_found'] else 'Keine erkannt'
        report.append(f"  Gefundene Abschnitte: {sections_str}")
        report.append(f"  Kontaktinformationen: {'✓' if parsing['has_contact_info'] else '✗'}")
        report.append("")

        # Parsed Information
        parsed_info = ats.get("parsed_information", {})
        if parsed_info:
            report.append("EXTRAHIERTE INFORMATIONEN:")
            report.append("-" * 40)
            report.append(f"  Name: {parsed_info.get('name', 'Nicht gefunden')}")
            report.append(f"  E-Mail: {parsed_info.get('email', 'Nicht gefunden')}")
            report.append(f"  Telefon: {parsed_info.get('phone', 'Nicht gefunden')}")
            report.append(f"  Berufserfahrung: {parsed_info.get('experience_years', 'Unbekannt')}")
            if parsed_info.get('skills'):
                report.append(f"  Kenntnisse: {', '.join(parsed_info['skills'][:10])}")
            if parsed_info.get('job_titles'):
                report.append(f"  Berufsbezeichnungen: {', '.join(parsed_info['job_titles'][:5])}")
            report.append("")

        # AGG Compliance Issues (German specific)
        agg_issues = ats.get("agg_compliance_issues", [])
        if agg_issues:
            report.append("⚠ AGG-KONFORMITÄTSPROBLEME GEFUNDEN:")
            report.append("-" * 40)
            report.append("WARNUNG: Folgende Angaben sind in Deutschland seit 2006 nicht erlaubt")
            report.append("(Allgemeines Gleichbehandlungsgesetz - Diskriminierungsschutz):")
            report.append("")
            for i, issue in enumerate(agg_issues, 1):
                severity_icon = {"high": "❌", "medium": "⚠", "low": "ℹ"}.get(issue.get("severity", "medium"), "⚠")
                report.append(f"  {severity_icon} {i}. {issue.get('issue')} (gefunden in: {issue.get('location')})")
                report.append(f"       Empfehlung: {issue.get('recommendation')}")
            report.append("")

        # Spelling Errors
        spelling_errors = ats.get("spelling_errors", [])
        if spelling_errors:
            report.append("RECHTSCHREIBFEHLER GEFUNDEN:")
            report.append("-" * 40)
            for i, error in enumerate(spelling_errors, 1):
                report.append(f"  {i}. '{error.get('word')}' in {error.get('location')}")
                report.append(f"     → Vorschlag: {error.get('suggestion')}")
            report.append("")

        # Grammar Errors
        grammar_errors = ats.get("grammar_errors", [])
        if grammar_errors:
            report.append("GRAMMATIKFEHLER GEFUNDEN:")
            report.append("-" * 40)
            for i, error in enumerate(grammar_errors, 1):
                report.append(f"  {i}. {error.get('issue')} in {error.get('location')}")
                report.append(f"     → Vorschlag: {error.get('suggestion')}")
            report.append("")

        # Keyword Analysis
        keyword_analysis = ats.get("keyword_analysis", {})
        if keyword_analysis and (keyword_analysis.get("overspecialized_terms") or
                                 keyword_analysis.get("missing_common_keywords")):
            report.append("KEYWORD-OPTIMIERUNG:")
            report.append("-" * 40)

            if keyword_analysis.get("overspecialized_terms"):
                report.append("  Überspezialisierte Begriffe (zu spezifisch/selten):")
                for term in keyword_analysis["overspecialized_terms"]:
                    alt = keyword_analysis.get("suggested_alternatives", {}).get(term, "N/A")
                    report.append(f"    • '{term}' → Erwägen Sie: '{alt}'")

            if keyword_analysis.get("missing_common_keywords"):
                report.append("  Fehlende gängige Keywords:")
                for keyword in keyword_analysis["missing_common_keywords"]:
                    report.append(f"    • {keyword}")
            report.append("")

        # Industry Parsing Issues (German)
        parsing_issues = ats.get("industry_parsing_issues", [])
        if parsing_issues:
            report.append("⚠ INDUSTRIE-STANDARD ATS-PARSING-PROBLEME:")
            report.append("-" * 40)
            report.append("Basierend auf echten ATS-Systemen (StepStone, XING, Indeed.de):")
            report.append("")
            for i, issue in enumerate(parsing_issues, 1):
                severity_icon = {"critical": "❌", "high": "⚠", "medium": "ℹ", "low": "•"}.get(issue.get("severity", "medium"), "ℹ")
                report.append(f"  {severity_icon} {i}. {issue.get('issue')} ({issue.get('severity', 'unknown').upper()})")
                report.append(f"       Position: {issue.get('location')}")
                report.append(f"       ATS-Auswirkung: {issue.get('ats_impact')}")
                report.append(f"       Punktabzug: -{issue.get('score_deduction', 0)} Punkte")
                report.append(f"       Behebung: {issue.get('recommendation')}")
            report.append("")

        # Date Consistency Issues
        date_issues = ats.get("date_consistency", {})
        if date_issues.get("has_issues"):
            report.append("⏰ DATUMSFORMAT-PROBLEME:")
            report.append("-" * 40)
            for problem in date_issues.get("problems", []):
                report.append(f"  • {problem}")
            report.append("")

        # Employment Gaps
        emp_gaps = ats.get("employment_gaps", [])
        if emp_gaps:
            unexplained = [g for g in emp_gaps if not g.get("explained", False)]
            if unexplained:
                report.append("📅 BESCHÄFTIGUNGSLÜCKEN (Unerklärte):")
                report.append("-" * 40)
                for gap in unexplained:
                    report.append(f"  • {gap.get('gap')}: {gap.get('duration')} Monate")
                report.append("  Empfehlung: Lücken erklären (z.B. 'Sabbatical', 'Weiterbildung', 'Elternzeit')")
                report.append("")

        # Strengths
        strengths = ats.get("strengths", [])
        if strengths:
            report.append("STÄRKEN:")
            report.append("-" * 40)
            for strength in strengths:
                report.append(f"  ✓ {strength}")
            report.append("")

        # Weaknesses
        weaknesses = ats.get("weaknesses", [])
        if weaknesses:
            report.append("SCHWÄCHEN:")
            report.append("-" * 40)
            for weakness in weaknesses:
                report.append(f"  ✗ {weakness}")
            report.append("")

        # Improvement Steps
        improvements = ats.get("improvement_steps", [])
        if improvements:
            report.append("VERBESSERUNGSEMPFEHLUNGEN:")
            report.append("-" * 40)
            for i, step in enumerate(improvements, 1):
                report.append(f"  {i}. {step}")
            report.append("")

        # ATS Reasoning
        reasoning = ats.get("ats_reasoning", "")
        if reasoning:
            report.append("DETAILLIERTE ANALYSE:")
            report.append("-" * 40)
            report.append(reasoning)
            report.append("")

        report.append("=" * 80)
        return "\n".join(report)
