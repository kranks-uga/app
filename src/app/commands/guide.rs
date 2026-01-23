use crate::app::guides::GuideRegistry;

/// Обработка команд работы с гайдами
pub fn process_guide_command(cmd: &str, guides: &GuideRegistry) -> Option<String> {
    // Список всех гайдов
    if cmd == "гайды" || cmd == "guides" || cmd == "обучение" {
        return Some(guides.format_list());
    }

    // Показать конкретный гайд: "гайд pacman" или "гайд wifi"
    if cmd.starts_with("гайд ") || cmd.starts_with("guide ") {
        let guide_id = cmd
            .trim_start_matches("гайд ")
            .trim_start_matches("guide ")
            .trim();

        if let Some(guide) = guides.get(guide_id) {
            return Some(guide.format());
        }

        // Поиск по ключевому слову если точный ID не найден
        let results = guides.search(guide_id);
        if results.is_empty() {
            return Some(format!(
                "Гайд '{}' не найден.\n\nИспользуйте 'гайды' для списка доступных.",
                guide_id
            ));
        } else if results.len() == 1 {
            return Some(results[0].format());
        } else {
            let mut output = format!(
                "Найдено {} гайдов по запросу '{}':\n\n",
                results.len(),
                guide_id
            );
            for guide in results {
                output.push_str(&format!("• {} — {}\n", guide.id, guide.title));
            }
            output.push_str("\nУточните запрос: гайд <название>");
            return Some(output);
        }
    }

    // Поиск гайдов
    if cmd.starts_with("найти гайд ") || cmd.starts_with("поиск гайдов ") {
        let query = cmd
            .trim_start_matches("найти гайд ")
            .trim_start_matches("поиск гайдов ")
            .trim();

        let results = guides.search(query);
        if results.is_empty() {
            return Some(format!("Гайды по запросу '{}' не найдены.", query));
        }

        let mut output = format!("Найдено {} гайдов:\n\n", results.len());
        for guide in results {
            output.push_str(&format!("• {} — {}\n", guide.id, guide.title));
        }
        output.push_str("\nИспользуйте: гайд <название>");
        return Some(output);
    }

    None
}
