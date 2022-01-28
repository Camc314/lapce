use druid::{
    kurbo::Line,
    piet::{Text, TextLayout, TextLayoutBuilder},
    BoxConstraints, Color, Command, Env, Event, EventCtx, FontFamily, LayoutCtx,
    LifeCycle, LifeCycleCtx, MouseEvent, PaintCtx, Point, Rect, RenderContext, Size,
    Target, UpdateCtx, Widget,
};

use crate::{
    command::{
        CommandTarget, LapceCommandNew, LapceWorkbenchCommand, LAPCE_NEW_COMMAND,
    },
    config::LapceTheme,
    data::LapceWindowData,
    state::LapceWorkspaceType,
    svg::get_svg,
};

pub struct Title {
    mouse_pos: Point,
    commands: Vec<(Rect, Command)>,
}

impl Title {
    pub fn new() -> Self {
        Self {
            mouse_pos: Point::ZERO,
            commands: Vec::new(),
        }
    }

    fn icon_hit_test(&self, mouse_event: &MouseEvent) -> bool {
        for (rect, _) in self.commands.iter() {
            if rect.contains(mouse_event.pos) {
                return true;
            }
        }
        false
    }

    fn mouse_down(&self, ctx: &mut EventCtx, mouse_event: &MouseEvent) {
        for (rect, command) in self.commands.iter() {
            if rect.contains(mouse_event.pos) {
                ctx.submit_command(command.clone());
            }
        }
    }
}

impl Widget<LapceWindowData> for Title {
    fn event(
        &mut self,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut LapceWindowData,
        env: &Env,
    ) {
        match event {
            Event::MouseMove(mouse_event) => {
                self.mouse_pos = mouse_event.pos;
                if self.icon_hit_test(mouse_event) {
                    ctx.set_cursor(&druid::Cursor::Pointer);
                    ctx.request_paint();
                } else {
                    ctx.clear_cursor();
                    ctx.request_paint();
                }
            }
            Event::MouseDown(mouse_event) => {
                self.mouse_down(ctx, mouse_event);
            }
            _ => {}
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &LapceWindowData,
        env: &Env,
    ) {
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx,
        old_data: &LapceWindowData,
        data: &LapceWindowData,
        env: &Env,
    ) {
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx,
        bc: &BoxConstraints,
        data: &LapceWindowData,
        env: &Env,
    ) -> Size {
        Size::new(bc.max().width, 28.0)
    }

    fn paint(&mut self, ctx: &mut PaintCtx, data: &LapceWindowData, env: &Env) {
        let size = ctx.size();
        let rect = size.to_rect();
        ctx.fill(
            rect,
            data.config
                .get_color_unchecked(LapceTheme::PANEL_BACKGROUND),
        );

        self.commands.clear();

        let mut x = 0.0;
        #[cfg(target_os = "macos")]
        let mut x = 70.0;

        let padding = 15.0;

        let tab = data.tabs.get(&data.active_id).unwrap();
        let remote_text = if let Some(workspace) = tab.workspace.as_ref() {
            match &workspace.kind {
                LapceWorkspaceType::Local => None,
                LapceWorkspaceType::RemoteSSH(_, host) => {
                    let text_layout = ctx
                        .text()
                        .new_text_layout(format!("SSH: {host}"))
                        .font(FontFamily::SYSTEM_UI, 13.0)
                        .text_color(
                            data.config
                                .get_color_unchecked(LapceTheme::EDITOR_BACKGROUND)
                                .clone(),
                        )
                        .build()
                        .unwrap();
                    Some(text_layout)
                }
            }
        } else {
            None
        };

        let remote_rect = Size::new(
            size.height
                + 10.0
                + remote_text
                    .as_ref()
                    .map(|t| t.size().width + padding - 5.0)
                    .unwrap_or(0.0),
            size.height,
        )
        .to_rect()
        .with_origin(Point::new(x, 0.0));
        ctx.fill(remote_rect, &Color::rgb8(64, 120, 242));
        let remote_svg = get_svg("remote.svg").unwrap();
        ctx.draw_svg(
            &remote_svg,
            remote_rect
                .with_origin(Point::new(x + 5.0, 0.0))
                .inflate(-5.0, -5.0),
            Some(
                data.config
                    .get_color_unchecked(LapceTheme::EDITOR_BACKGROUND),
            ),
        );
        if let Some(text_layout) = remote_text.as_ref() {
            ctx.draw_text(
                text_layout,
                Point::new(
                    x + size.height + 5.0,
                    (size.height - text_layout.size().height) / 2.0,
                ),
            );
        }
        x += remote_rect.width();

        let command_rect = Size::ZERO.to_rect().with_origin(Point::new(x, 0.0));

        x += 5.0;
        let folder_svg = get_svg("default_folder.svg").unwrap();
        let folder_rect = Size::new(size.height, size.height)
            .to_rect()
            .with_origin(Point::new(x, 0.0));
        ctx.draw_svg(
            &folder_svg,
            folder_rect.inflate(-5.0, -5.0),
            Some(
                data.config
                    .get_color_unchecked(LapceTheme::EDITOR_FOREGROUND),
            ),
        );
        x += size.height;
        let text = if let Some(workspace) = tab.workspace.as_ref() {
            workspace
                .path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        } else {
            "Open Folder".to_string()
        };
        let text_layout = ctx
            .text()
            .new_text_layout(text)
            .font(FontFamily::SYSTEM_UI, 13.0)
            .text_color(
                data.config
                    .get_color_unchecked(LapceTheme::EDITOR_FOREGROUND)
                    .clone(),
            )
            .build()
            .unwrap();
        ctx.draw_text(
            &text_layout,
            Point::new(x, (size.height - text_layout.size().height) / 2.0),
        );
        x += text_layout.size().width + padding;
        self.commands.push((
            command_rect.with_size(Size::new(x - command_rect.x0, size.height)),
            Command::new(
                LAPCE_NEW_COMMAND,
                LapceCommandNew {
                    cmd: LapceWorkbenchCommand::OpenFolder.to_string(),
                    data: None,
                    palette_desc: Some("Open Folder".to_string()),
                    target: CommandTarget::Workbench,
                },
                Target::Widget(tab.id),
            ),
        ));

        let line_color = data.config.get_color_unchecked(LapceTheme::LAPCE_BORDER);
        let line = Line::new(Point::new(x, 0.0), Point::new(x, size.height));
        ctx.stroke(line, line_color, 1.0);
    }
}