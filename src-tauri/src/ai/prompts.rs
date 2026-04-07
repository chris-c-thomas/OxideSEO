//! Prompt templates for AI analysis features.
//!
//! Each function builds a `CompletionRequest` for a specific analysis type.
//! Content is truncated to fit within typical context windows.

use super::provider::{CompletionRequest, ResponseFormat};

/// Maximum words to include in prompts to stay within context limits.
const MAX_PROMPT_WORDS: usize = 4000;

/// Truncate text to approximately `max_words` words.
fn truncate_to_words(text: &str, max_words: usize) -> String {
    let words: Vec<&str> = text.split_whitespace().collect();
    if words.len() <= max_words {
        text.to_string()
    } else {
        let mut result = words[..max_words].join(" ");
        result.push_str(" [... truncated]");
        result
    }
}

/// Build a content quality scoring request.
pub fn content_quality_request(
    page_text: &str,
    url: &str,
    title: Option<&str>,
) -> CompletionRequest {
    let truncated = truncate_to_words(page_text, MAX_PROMPT_WORDS);

    let system_prompt = r#"You are an SEO content quality analyst. Evaluate the provided web page content and return a JSON object with the following structure:

{
  "overallScore": <0-100>,
  "relevanceScore": <0-100>,
  "readabilityScore": <0-100>,
  "depthScore": <0-100>,
  "reasoning": "<brief explanation>",
  "suggestions": ["<improvement 1>", "<improvement 2>", ...]
}

Score criteria:
- relevanceScore: How well the content matches the page title/topic. Is it focused?
- readabilityScore: Sentence length, vocabulary level, structure. Lower = harder to read.
- depthScore: Does the content provide substantive information, or is it thin/generic?
- overallScore: Weighted average (relevance 30%, readability 30%, depth 40%)"#;

    let title_str = title.unwrap_or("(no title)");
    let user_prompt = format!("URL: {url}\nTitle: {title_str}\n\nPage content:\n{truncated}");

    CompletionRequest {
        system_prompt: system_prompt.into(),
        user_prompt,
        max_tokens: 500,
        temperature: 0.3,
        response_format: ResponseFormat::Json,
    }
}

/// Build a meta description generation request.
pub fn meta_description_request(
    page_text: &str,
    title: Option<&str>,
    current_desc: Option<&str>,
) -> CompletionRequest {
    let truncated = truncate_to_words(page_text, MAX_PROMPT_WORDS);

    let system_prompt = r#"You are an SEO specialist. Generate an optimized meta description for the given web page. Return a JSON object:

{
  "suggested": "<the meta description, 150-160 characters>",
  "charCount": <number>,
  "reasoning": "<why this description works>"
}

Guidelines:
- Target 150-160 characters (including spaces)
- Include the primary keyword naturally
- Write a compelling call-to-action or value proposition
- Avoid generic phrases like "Learn more" or "Click here""#;

    let title_str = title.unwrap_or("(no title)");
    let current = current_desc
        .map(|d| format!("\nCurrent meta description: {d}"))
        .unwrap_or_default();
    let user_prompt = format!("Title: {title_str}{current}\n\nPage content:\n{truncated}");

    CompletionRequest {
        system_prompt: system_prompt.into(),
        user_prompt,
        max_tokens: 300,
        temperature: 0.5,
        response_format: ResponseFormat::Json,
    }
}

/// Build a title tag suggestion request.
pub fn title_tag_request(
    page_text: &str,
    current_title: Option<&str>,
    url: &str,
) -> CompletionRequest {
    let truncated = truncate_to_words(page_text, MAX_PROMPT_WORDS);

    let system_prompt = r#"You are an SEO specialist. Suggest improved title tags for the given page. Return a JSON object:

{
  "suggestions": [
    {"title": "<suggestion>", "charCount": <number>},
    {"title": "<suggestion>", "charCount": <number>},
    {"title": "<suggestion>", "charCount": <number>}
  ],
  "reasoning": "<why these titles work>"
}

Guidelines:
- Target 50-60 characters each
- Put the primary keyword near the beginning
- Make each suggestion distinct in approach (keyword-focused, benefit-focused, question-based)
- Include the brand name at the end if space permits"#;

    let current = current_title
        .map(|t| format!("\nCurrent title: {t}"))
        .unwrap_or_default();
    let user_prompt = format!("URL: {url}{current}\n\nPage content:\n{truncated}");

    CompletionRequest {
        system_prompt: system_prompt.into(),
        user_prompt,
        max_tokens: 400,
        temperature: 0.5,
        response_format: ResponseFormat::Json,
    }
}

/// Build a crawl summary generation request.
pub fn crawl_summary_request(stats_json: &str) -> CompletionRequest {
    let system_prompt = r#"You are an SEO audit specialist. Given the crawl statistics below, generate an executive summary. Return a JSON object:

{
  "summary": "<2-3 paragraph natural language summary>",
  "topActions": [
    "<action item 1>",
    "<action item 2>",
    "<action item 3>",
    "<action item 4>",
    "<action item 5>"
  ],
  "overallHealth": "<good|fair|poor>",
  "criticalIssuesCount": <number>,
  "keyFindings": ["<finding 1>", "<finding 2>", "<finding 3>"]
}

Focus on:
- The most impactful issues (errors first, then warnings)
- Pages with the worst scores
- Patterns across the site (e.g., "40% of pages are missing meta descriptions")
- Actionable next steps prioritized by impact"#;

    CompletionRequest {
        system_prompt: system_prompt.into(),
        user_prompt: format!("Crawl statistics:\n{stats_json}"),
        max_tokens: 800,
        temperature: 0.3,
        response_format: ResponseFormat::Json,
    }
}

/// Build a structured data recommendation request.
pub fn structured_data_request(
    page_text: &str,
    url: &str,
    title: Option<&str>,
) -> CompletionRequest {
    let truncated = truncate_to_words(page_text, MAX_PROMPT_WORDS);

    let system_prompt = r#"You are an SEO structured data specialist. Analyze the web page and recommend appropriate JSON-LD structured data. Return a JSON object:

{
  "pageType": "<detected page type: article, product, faq, local_business, recipe, event, organization, person, other>",
  "recommendedSchemas": [
    {
      "type": "<Schema.org type, e.g. Article, Product, FAQPage>",
      "priority": "<high|medium|low>",
      "reason": "<why this schema fits>",
      "example": "<minimal JSON-LD snippet>"
    }
  ],
  "existingMarkup": "<description of any existing structured data found, or 'none'>",
  "reasoning": "<overall analysis>"
}

Guidelines:
- Identify the primary content type from the page text and URL structure
- Recommend the most impactful Schema.org types (max 3)
- Prioritize schemas that enable rich results in Google Search
- Include minimal but valid JSON-LD examples"#;

    let title_str = title.unwrap_or("(no title)");
    let user_prompt = format!("URL: {url}\nTitle: {title_str}\n\nPage content:\n{truncated}");

    CompletionRequest {
        system_prompt: system_prompt.into(),
        user_prompt,
        max_tokens: 800,
        temperature: 0.3,
        response_format: ResponseFormat::Json,
    }
}

/// Build an accessibility narrative request.
pub fn accessibility_request(
    page_text: &str,
    url: &str,
    title: Option<&str>,
    h1s: &[String],
    images_without_alt: u32,
) -> CompletionRequest {
    let truncated = truncate_to_words(page_text, MAX_PROMPT_WORDS);

    let system_prompt = r#"You are a web accessibility specialist following WCAG 2.1 guidelines. Analyze the page context and provide plain-English accessibility remediation guidance. Return a JSON object:

{
  "overallRating": "<good|fair|poor>",
  "issues": [
    {
      "wcagCriterion": "<e.g. 1.1.1 Non-text Content>",
      "level": "<A|AA|AAA>",
      "description": "<plain-English description of the issue>",
      "remediation": "<specific, actionable fix instructions>",
      "priority": "<high|medium|low>"
    }
  ],
  "positives": ["<things the page does well>"],
  "reasoning": "<overall accessibility assessment>"
}

Focus on:
- Image alt text completeness
- Heading hierarchy and document structure
- Link text descriptiveness
- Color contrast and readability considerations
- Keyboard navigation implications"#;

    let title_str = title.unwrap_or("(no title)");
    let h1_list = if h1s.is_empty() {
        "none".to_string()
    } else {
        h1s.join(", ")
    };
    let user_prompt = format!(
        "URL: {url}\nTitle: {title_str}\nH1 headings: {h1_list}\nImages without alt text: {images_without_alt}\n\nPage content:\n{truncated}"
    );

    CompletionRequest {
        system_prompt: system_prompt.into(),
        user_prompt,
        max_tokens: 800,
        temperature: 0.3,
        response_format: ResponseFormat::Json,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_truncate_to_words_short_text() {
        let text = "Hello world this is short";
        assert_eq!(truncate_to_words(text, 100), text);
    }

    #[test]
    fn test_truncate_to_words_long_text() {
        let words: Vec<&str> = (0..100).map(|_| "word").collect();
        let text = words.join(" ");
        let result = truncate_to_words(&text, 10);
        assert!(result.ends_with("[... truncated]"));
        // 10 words + truncated marker
        assert_eq!(
            result.split_whitespace().count(),
            10 + 2 // "[..." and "truncated]"
        );
    }

    #[test]
    fn test_content_quality_request_format() {
        let req = content_quality_request(
            "Some page content here",
            "https://example.com",
            Some("Example"),
        );
        assert!(!req.system_prompt.is_empty());
        assert!(req.user_prompt.contains("https://example.com"));
        assert!(req.user_prompt.contains("Example"));
        assert_eq!(req.response_format, ResponseFormat::Json);
    }

    #[test]
    fn test_meta_description_request_without_current() {
        let req = meta_description_request("Page content", Some("Title"), None);
        assert!(!req.user_prompt.contains("Current meta description"));
    }

    #[test]
    fn test_meta_description_request_with_current() {
        let req = meta_description_request("Page content", Some("Title"), Some("Old desc"));
        assert!(
            req.user_prompt
                .contains("Current meta description: Old desc")
        );
    }

    #[test]
    fn test_structured_data_request_format() {
        let req = structured_data_request(
            "Page about recipes",
            "https://example.com/recipe",
            Some("Best Pasta"),
        );
        assert!(req.system_prompt.contains("structured data"));
        assert!(req.user_prompt.contains("Best Pasta"));
        assert!(req.user_prompt.contains("recipe"));
        assert_eq!(req.response_format, ResponseFormat::Json);
    }

    #[test]
    fn test_accessibility_request_format() {
        let req = accessibility_request(
            "Page content here",
            "https://example.com",
            Some("Home"),
            &["Welcome".into()],
            3,
        );
        assert!(req.system_prompt.contains("WCAG"));
        assert!(req.user_prompt.contains("Images without alt text: 3"));
        assert!(req.user_prompt.contains("Welcome"));
        assert_eq!(req.response_format, ResponseFormat::Json);
    }

    #[test]
    fn test_accessibility_request_no_h1s() {
        let req = accessibility_request("content", "https://example.com", None, &[], 0);
        assert!(req.user_prompt.contains("H1 headings: none"));
    }

    #[test]
    fn test_crawl_summary_request_format() {
        let req = crawl_summary_request(r#"{"pages": 100, "errors": 5}"#);
        assert!(req.user_prompt.contains("Crawl statistics"));
        assert_eq!(req.response_format, ResponseFormat::Json);
    }
}
