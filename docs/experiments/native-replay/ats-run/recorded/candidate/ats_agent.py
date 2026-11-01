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


GERMAN = {
    "ATS CV COMPATIBILITY REPORT": "ATS LEBENSLAUF-KOMPATIBILITÄTSBERICHT",
    "OVERALL ATS SCORE": "GESAMT-ATS-BEWERTUNG",
    "CATEGORY SCORES:": "KATEGORIE-BEWERTUNGEN:",
    "PARSING SUMMARY:": "PARSING-ZUSAMMENFASSUNG:",
    "Pages": "Seiten",
    "Words": "Wörter",
    "Sections Found": "Gefundene Abschnitte",
    "None detected": "Keine erkannt",
    "Contact Info": "Kontaktinformationen",
    "EXTRACTED INFORMATION:": "EXTRAHIERTE INFORMATIONEN:",
    "Email": "E-Mail",
    "Phone": "Telefon",
    "Experience": "Berufserfahrung",
    "Not found": "Nicht gefunden",
    "Unknown": "Unbekannt",
    "Skills": "Kenntnisse",
    "Job Titles": "Berufsbezeichnungen",
    "SPELLING ERRORS FOUND:": "RECHTSCHREIBFEHLER GEFUNDEN:",
    "GRAMMAR ERRORS FOUND:": "GRAMMATIKFEHLER GEFUNDEN:",
    "Suggestion": "Vorschlag",
    "KEYWORD OPTIMIZATION:": "KEYWORD-OPTIMIERUNG:",
    "  Overspecialized terms (too niche):": "  Überspezialisierte Begriffe (zu spezifisch/selten):",
    "Consider": "Erwägen Sie",
    "  Missing common keywords:": "  Fehlende gängige Keywords:",
    "⚠ INDUSTRY ATS PARSING ISSUES:": "⚠ INDUSTRIE-STANDARD ATS-PARSING-PROBLEME:",
    "Based on real ATS systems (Ashby, Greenhouse, Workday, Lever):": "Basierend auf echten ATS-Systemen (StepStone, XING, Indeed.de):",
    "Location": "Position",
    "ATS Impact": "ATS-Auswirkung",
    "Score Deduction": "Punktabzug",
    "points": "Punkte",
    "Fix": "Behebung",
    "⏰ DATE FORMAT ISSUES:": "⏰ DATUMSFORMAT-PROBLEME:",
    "📅 EMPLOYMENT GAPS (Unexplained):": "📅 BESCHÄFTIGUNGSLÜCKEN (Unerklärte):",
    "months": "Monate",
    "  Recommendation: Add explanations (e.g., 'Sabbatical', 'Professional Development')": "  Empfehlung: Lücken erklären (z.B. 'Sabbatical', 'Weiterbildung', 'Elternzeit')",
    "STRENGTHS:": "STÄRKEN:",
    "WEAKNESSES:": "SCHWÄCHEN:",
    "IMPROVEMENT RECOMMENDATIONS:": "VERBESSERUNGSEMPFEHLUNGEN:",
    "DETAILED ANALYSIS:": "DETAILLIERTE ANALYSE:",
}

CATEGORY_NAMES_DE = {
    "keywords": "Schlüsselwörter",
    "structure": "Struktur",
    "parsing_capability": "Parsing-Fähigkeit",
    "content_quality": "Inhaltsqualität",
    "spelling_grammar": "Rechtschreibung/Grammatik",
    "agg_compliance": "AGG-Konformität",
    "context_match": "Kontext-Übereinstimmung",
}


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
        """Dispatch the existing public report operation, including error behavior."""
        if not analysis_result.get("success"):
            error_msg = analysis_result.get('error', 'Unknown error')
            return f"ERROR: {error_msg}" if language != "de" else f"FEHLER: {error_msg}"
        if language is None:
            language = analysis_result.get("language", "en")
        ats = analysis_result["ats_analysis"]
        parsing = analysis_result["parsing_summary"]
        if language == "de":
            return self._generate_german_report(ats, parsing)
        else:
            return self._generate_english_report(ats, parsing)

    def _generate_english_report(self, ats: dict, parsing: dict) -> str:
        return self._render_report(ats, parsing, german=False)

    def _generate_german_report(self, ats: dict, parsing: dict) -> str:
        return self._render_report(ats, parsing, german=True)

    def _render_report(self, ats: dict, parsing: dict, *, german: bool) -> str:
        """One section sequence; translations are data, German-only behavior explicit."""
        def label(english: str) -> str:
            return GERMAN.get(english, english) if german else english

        report = ["=" * 80, label("ATS CV COMPATIBILITY REPORT"), "=" * 80, "",
                  f"{label('OVERALL ATS SCORE')}: {ats.get('overall_score', 'N/A')}/100", ""]
        if german:
            report.extend(["Erkannte Sprache: Deutsch", ""])
        elif parsing.get("detected_language"):
            lang_display = "German" if parsing["detected_language"] == "de" else parsing["detected_language"].upper()
            report.extend([f"Detected Language: {lang_display}", ""])
        report.extend([label("CATEGORY SCORES:"), "-" * 40])
        for category, score in ats.get("category_scores", {}).items():
            if score is not None:
                bar = "█" * (score // 5) + "░" * (20 - score // 5)
                category_name = category.replace('_', ' ').title()
                if german:
                    category_name = CATEGORY_NAMES_DE.get(category, category_name)
                width = 25 if german else 20
                report.append(f"  {category_name:{width}s}: {score:3d}/100 [{bar}]")
        report.extend(["", label("PARSING SUMMARY:"), "-" * 40,
                       f"  {label('Pages')}: {parsing['page_count']}",
                       f"  {label('Words')}: {parsing['word_count']}"])
        sections = ', '.join(parsing['sections_found']) if parsing['sections_found'] else label('None detected')
        report.extend([f"  {label('Sections Found')}: {sections}",
                       f"  {label('Contact Info')}: {'✓' if parsing['has_contact_info'] else '✗'}", ""])
        parsed_info = ats.get("parsed_information", {})
        if parsed_info:
            report.extend([label("EXTRACTED INFORMATION:"), "-" * 40,
                f"  Name: {parsed_info.get('name', label('Not found'))}",
                f"  {label('Email')}: {parsed_info.get('email', label('Not found'))}",
                f"  {label('Phone')}: {parsed_info.get('phone', label('Not found'))}",
                f"  {label('Experience')}: {parsed_info.get('experience_years', label('Unknown'))}"])
            if parsed_info.get('skills'):
                report.append(f"  {label('Skills')}: {', '.join(parsed_info['skills'][:10])}")
            if parsed_info.get('job_titles'):
                report.append(f"  {label('Job Titles')}: {', '.join(parsed_info['job_titles'][:5])}")
            report.append("")
        if german:
            agg_issues = ats.get("agg_compliance_issues", [])
            if agg_issues:
                report.extend(["⚠ AGG-KONFORMITÄTSPROBLEME GEFUNDEN:", "-" * 40,
                    "WARNUNG: Folgende Angaben sind in Deutschland seit 2006 nicht erlaubt",
                    "(Allgemeines Gleichbehandlungsgesetz - Diskriminierungsschutz):", ""])
                for i, issue in enumerate(agg_issues, 1):
                    severity_icon = {"high": "❌", "medium": "⚠", "low": "ℹ"}.get(issue.get("severity", "medium"), "⚠")
                    report.extend([f"  {severity_icon} {i}. {issue.get('issue')} (gefunden in: {issue.get('location')})",
                                   f"       Empfehlung: {issue.get('recommendation')}"])
                report.append("")
        spelling_errors = ats.get("spelling_errors", [])
        if spelling_errors:
            report.extend([label("SPELLING ERRORS FOUND:"), "-" * 40])
            for i, error in enumerate(spelling_errors, 1):
                report.extend([f"  {i}. '{error.get('word')}' in {error.get('location')}",
                               f"     → {label('Suggestion')}: {error.get('suggestion')}"])
            report.append("")
        grammar_errors = ats.get("grammar_errors", [])
        if grammar_errors:
            report.extend([label("GRAMMAR ERRORS FOUND:"), "-" * 40])
            for i, error in enumerate(grammar_errors, 1):
                report.extend([f"  {i}. {error.get('issue')} in {error.get('location')}",
                               f"     → {label('Suggestion')}: {error.get('suggestion')}"])
            report.append("")
        keyword_analysis = ats.get("keyword_analysis", {})
        if keyword_analysis and (keyword_analysis.get("overspecialized_terms") or
                                 keyword_analysis.get("missing_common_keywords")):
            report.extend([label("KEYWORD OPTIMIZATION:"), "-" * 40])
            if keyword_analysis.get("overspecialized_terms"):
                report.append(label("  Overspecialized terms (too niche):"))
                for term in keyword_analysis["overspecialized_terms"]:
                    alternative = keyword_analysis.get("suggested_alternatives", {}).get(term, "N/A")
                    report.append(f"    • '{term}' → {label('Consider')}: '{alternative}'")
            if keyword_analysis.get("missing_common_keywords"):
                report.append(label("  Missing common keywords:"))
                for keyword in keyword_analysis["missing_common_keywords"]:
                    report.append(f"    • {keyword}")
            report.append("")
        parsing_issues = ats.get("industry_parsing_issues", [])
        if parsing_issues:
            report.extend([label("⚠ INDUSTRY ATS PARSING ISSUES:"), "-" * 40,
                           label("Based on real ATS systems (Ashby, Greenhouse, Workday, Lever):"), ""])
            for i, issue in enumerate(parsing_issues, 1):
                severity_icon = {"critical": "❌", "high": "⚠", "medium": "ℹ", "low": "•"}.get(issue.get("severity", "medium"), "ℹ")
                report.extend([f"  {severity_icon} {i}. {issue.get('issue')} ({issue.get('severity', 'unknown').upper()})",
                    f"       {label('Location')}: {issue.get('location')}",
                    f"       {label('ATS Impact')}: {issue.get('ats_impact')}",
                    f"       {label('Score Deduction')}: -{issue.get('score_deduction', 0)} {label('points')}",
                    f"       {label('Fix')}: {issue.get('recommendation')}"])
            report.append("")
        date_issues = ats.get("date_consistency", {})
        if date_issues.get("has_issues"):
            report.extend([label("⏰ DATE FORMAT ISSUES:"), "-" * 40])
            for problem in date_issues.get("problems", []):
                report.append(f"  • {problem}")
            report.append("")
        emp_gaps = ats.get("employment_gaps", [])
        if emp_gaps:
            unexplained = [gap for gap in emp_gaps if not gap.get("explained", False)]
            if unexplained:
                report.extend([label("📅 EMPLOYMENT GAPS (Unexplained):"), "-" * 40])
                for gap in unexplained:
                    report.append(f"  • {gap.get('gap')}: {gap.get('duration')} {label('months')}")
                report.extend([label("  Recommendation: Add explanations (e.g., 'Sabbatical', 'Professional Development')"), ""])
        strengths = ats.get("strengths", [])
        if strengths:
            report.extend([label("STRENGTHS:"), "-" * 40])
            for strength in strengths:
                report.append(f"  ✓ {strength}")
            report.append("")
        weaknesses = ats.get("weaknesses", [])
        if weaknesses:
            report.extend([label("WEAKNESSES:"), "-" * 40])
            for weakness in weaknesses:
                report.append(f"  ✗ {weakness}")
            report.append("")
        improvements = ats.get("improvement_steps", [])
        if improvements:
            report.extend([label("IMPROVEMENT RECOMMENDATIONS:"), "-" * 40])
            for i, step in enumerate(improvements, 1):
                report.append(f"  {i}. {step}")
            report.append("")
        reasoning = ats.get("ats_reasoning", "")
        if reasoning:
            report.extend([label("DETAILED ANALYSIS:"), "-" * 40, reasoning, ""])
        report.append("=" * 80)
        return "\n".join(report)
