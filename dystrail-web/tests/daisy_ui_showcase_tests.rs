use dystrail_web::components::daisy_ui::foundation::{attr_value, class_list};
use dystrail_web::components::daisy_ui::*;
use futures::executor::block_on;
use yew::LocalServerRenderer;
use yew::prelude::*;

#[function_component(AtomsShowcase)]
fn atoms_showcase() -> Html {
    let select_options = vec![
        SelectOption {
            label: "One".into(),
            value: "one".into(),
            disabled: false,
        },
        SelectOption {
            label: "Two".into(),
            value: "two".into(),
            disabled: true,
        },
    ];

    html! {
        <div>
            <Alert title={Some(AttrValue::from("Heads up"))} message={Some(AttrValue::from("All systems nominal"))} />
            <Alert title={Some(AttrValue::from("Warning"))} variant={Some(DaisyColor::Warning)}>
                <span>{ "Check the gauges" }</span>
            </Alert>
            <Avatar initials={Some(AttrValue::from("DT"))} />
            <Avatar
                src={Some(AttrValue::from("https://example.com/avatar.png"))}
                alt={Some(AttrValue::from("Avatar"))}
                size={Some(DaisySize::Lg)}
            />
            <Badge label={Some(AttrValue::from("New"))} />
            <Badge label={Some(AttrValue::from("Info"))} variant={Some(DaisyColor::Info)} />
            <Badge>{"Inline badge"}</Badge>
            <Button label={Some(AttrValue::from("Click me"))} />
            <Button
                label={Some(AttrValue::from("Outline"))}
                variant={Some(DaisyColor::Secondary)}
                size={Some(DaisySize::Sm)}
                outline={true}
                aria_label={Some(AttrValue::from("Outline button"))}
            />
            <Button><span>{"Child button"}</span></Button>
            <Checkbox label={Some(AttrValue::from("Accept terms"))} checked={true} />
            <Indicator indicator={Some(html! { <span class="badge">{"1"}</span> })}>
                <span>{"Inbox"}</span>
            </Indicator>
            <Input value={AttrValue::from("user")} placeholder={Some(AttrValue::from("Name"))} />
            <Kbd keys={vec![AttrValue::from("Ctrl"), AttrValue::from("K")]} />
            <Label for_input={Some(AttrValue::from("field"))} text={Some(AttrValue::from("Field label"))} />
            <Link href={AttrValue::from("#")} label={Some(AttrValue::from("Go"))} />
            <Link href={AttrValue::from("https://example.com")} label={Some(AttrValue::from("External"))} new_tab={true} />
            <Link href="#">{ "Child link" }</Link>
            <Loading label={Some(AttrValue::from("Loading"))} />
            <Loading label={Some(AttrValue::from("Loading md"))} size={Some(DaisySize::Md)} />
            <Mask shape={Some(AttrValue::from("mask-squircle"))}>
                <span>{"Masked"}</span>
            </Mask>
            <Progress value={42.0} max={100.0} />
            <Progress value={70.0} max={100.0} label={Some(AttrValue::from("70%"))} />
            <RadialProgress value={75.0} label={Some(AttrValue::from("Charge"))} />
            <Radio name={AttrValue::from("group")} value={AttrValue::from("a")} label={Some(AttrValue::from("Option A"))} checked={true} />
            <Range value={50.0} />
            <Rating value={3} max={5} />
            <Select options={select_options} value={Some(AttrValue::from("one"))} />
            <Skeleton text={true} width={Some(AttrValue::from("8rem"))} />
            <Skeleton height={Some(AttrValue::from("2rem"))} />
            <Status variant={DaisyColor::Success} label={Some(AttrValue::from("Online"))} />
            <Status variant={DaisyColor::Info} size={Some(DaisySize::Sm)} label={Some(AttrValue::from("Idle"))} />
            <Textarea value={AttrValue::from("notes")} />
            <Toggle checked={true} label={Some(AttrValue::from("Toggle me"))} />
        </div>
    }
}

#[function_component(MoleculesShowcase)]
fn molecules_showcase() -> Html {
    let crumbs = vec![
        Crumb {
            label: AttrValue::from("Home"),
            href: Some(AttrValue::from("#")),
        },
        Crumb {
            label: AttrValue::from("Settings"),
            href: None,
        },
    ];
    let menu_items = vec![
        MenuItem {
            label: AttrValue::from("Overview"),
            href: Some(AttrValue::from("#")),
            active: true,
            disabled: false,
        },
        MenuItem {
            label: AttrValue::from("Billing"),
            href: None,
            active: false,
            disabled: true,
        },
    ];
    let stat_items = vec![StatItem {
        title: AttrValue::from("Health"),
        value: AttrValue::from("98%"),
        description: Some(AttrValue::from("Stable")),
        figure: Some(html! { <span>{"♥"}</span> }),
        actions: Some(html! { <button class="btn btn-xs">{"Details"}</button> }),
    }];
    let tabs = vec![
        TabItem {
            id: AttrValue::from("tab-a"),
            label: AttrValue::from("Tab A"),
            disabled: false,
            content: Some(html! { <div>{"Tab A content"}</div> }),
        },
        TabItem {
            id: AttrValue::from("tab-b"),
            label: AttrValue::from("Tab B"),
            disabled: false,
            content: Some(html! { <div>{"Tab B content"}</div> }),
        },
    ];
    let filter_options = vec![
        FilterOption {
            label: AttrValue::from("Active"),
            value: AttrValue::from("active"),
        },
        FilterOption {
            label: AttrValue::from("Paused"),
            value: AttrValue::from("paused"),
        },
    ];
    let dropdown_menu = Children::new(vec![
        html! { <li role="menuitem">{"First"}</li> },
        html! { <li role="menuitem">{"Second"}</li> },
    ]);

    html! {
        <div>
            <Accordion title={AttrValue::from("Details")} open={true}>
                <p>{"Accordion body"}</p>
            </Accordion>
            <Breadcrumbs items={crumbs} />
            <Calendar year={2024} month={2} selected_day={Some(29)} />
            <Card title={Some(AttrValue::from("Card title"))} subtitle={Some(AttrValue::from("Subtitle"))}>
                <p>{"Card body"}</p>
            </Card>
            <Chat author={AttrValue::from("Operator")} message={AttrValue::from("Ping")} timestamp={Some(AttrValue::from("10:12"))} />
            <Collapse title={AttrValue::from("More")} open={true}>
                <p>{"Collapse body"}</p>
            </Collapse>
            <Countdown millis={90_000} label={Some(AttrValue::from("ETA"))} />
            <Diff before={AttrValue::from("Before")} after={AttrValue::from("After")} caption={Some(AttrValue::from("Delta"))} />
            <Divider text={Some(AttrValue::from("Or"))} />
            <Dropdown label={AttrValue::from("Menu")} open={Some(true)} menu={dropdown_menu} />
            <Fab label={Some(AttrValue::from("Create"))} />
            <Fieldset legend={Some(AttrValue::from("Profile"))}>
                <Input value={AttrValue::from("Name")} />
            </Fieldset>
            <FileInput label={Some(AttrValue::from("Upload"))} />
            <Filter options={filter_options} selected={vec![AttrValue::from("active")]} />
            <HoverGallery>{ "Hover gallery" }</HoverGallery>
            <Hover3d>{ "Hover3d" }</Hover3d>
            <Join>
                <Button label={Some(AttrValue::from("Left"))} />
                <Button label={Some(AttrValue::from("Right"))} />
            </Join>
            <List items={vec![AttrValue::from("Alpha"), AttrValue::from("Beta")]} />
            <List>
                <li>{"Child list item"}</li>
            </List>
            <Menu items={menu_items} />
            <Menu items={Vec::new()}>
                <li>{"Child menu item"}</li>
            </Menu>
            <Stack align_top={true}>
                <div class="card">{"Stacked 1"}</div>
                <div class="card">{"Stacked 2"}</div>
            </Stack>
            <Stat items={stat_items} />
            <Steps steps={vec![AttrValue::from("One"), AttrValue::from("Two"), AttrValue::from("Three")]} current={1} horizontal={true} />
            <Swap on={html!{"On"}} off={html!{"Off"}} indeterminate={Some(html!{"Maybe"})} />
            <Tab tabs={tabs} active_id={Some(AttrValue::from("tab-b"))} />
            <TextRotate items={vec![AttrValue::from("Alpha"), AttrValue::from("Beta")]} active_index={Some(1)} />
            <TextRotate items={Vec::new()} />
            <Tooltip text={AttrValue::from("Tooltip")}>
                <span>{"Hover me"}</span>
            </Tooltip>
            <Validator state={Some(ValidationState::Success)} message={Some(AttrValue::from("Looks good"))}>
                <Input value={AttrValue::from("ok")} />
            </Validator>
        </div>
    }
}

#[function_component(MoleculesVariants)]
fn molecules_variants() -> Html {
    let tabs = vec![
        TabItem {
            id: AttrValue::from("tab-1"),
            label: AttrValue::from("Enabled"),
            disabled: false,
            content: Some(html! { <div>{"Enabled tab"}</div> }),
        },
        TabItem {
            id: AttrValue::from("tab-2"),
            label: AttrValue::from("Disabled"),
            disabled: true,
            content: Some(html! { <div>{"Disabled tab"}</div> }),
        },
    ];
    let dropdown_menu = Children::new(vec![html! { <li role="menuitem">{"Only"}</li> }]);

    html! {
        <div>
            <Dropdown label={AttrValue::from("End")} align_end={true} menu={dropdown_menu.clone()} />
            <Dropdown label={AttrValue::from("Closed")} open={Some(false)} menu={dropdown_menu} />
            <FileInput accept={Some(AttrValue::from("image/*"))} multiple={true} />
            <Tab tabs={tabs} />
            <TextRotate items={vec![AttrValue::from("Solo")]} active_index={Some(5)} />
        </div>
    }
}

#[function_component(OrganismsShowcase)]
fn organisms_showcase() -> Html {
    let toast_items = vec![ToastItem {
        id: AttrValue::from("toast-1"),
        content: html! { <span>{"Saved"}</span> },
    }];
    let timeline_items = vec![
        TimelineItem {
            title: AttrValue::from("Start"),
            content: Some(AttrValue::from("Boot")),
        },
        TimelineItem {
            title: AttrValue::from("Finish"),
            content: Some(AttrValue::from("Done")),
        },
    ];
    let table_headers = vec![AttrValue::from("Col A"), AttrValue::from("Col B")];
    let table_rows = vec![
        vec![AttrValue::from("A1"), AttrValue::from("B1")],
        vec![AttrValue::from("A2"), AttrValue::from("B2")],
    ];

    html! {
        <div>
            <Navbar
                brand={Some(html! { <span class="font-bold">{"Brand"}</span> })}
                start={Some(html! { <a href="#">{"Start"}</a> })}
                center={Some(html! { <span>{"Center"}</span> })}
                end={Some(html! { <button class="btn btn-xs">{"Sign in"}</button> })}
            />
            <Drawer open={true} side={html! { <nav>{"Side nav"}</nav> }}>
                <div>{"Drawer content"}</div>
            </Drawer>
            <Footer>{"Footer content"}</Footer>
            <Hero title={AttrValue::from("Hero Title")} subtitle={Some(AttrValue::from("Subtitle"))}>
                <p>{"Hero body"}</p>
            </Hero>
            <Modal open={true} title={AttrValue::from("Dialog")} description={Some(AttrValue::from("Details"))}>
                <p>{"Modal content"}</p>
            </Modal>
            <Pagination total_pages={3} current_page={1} />
            <Table headers={table_headers} rows={table_rows} />
            <ThemeController themes={vec![AttrValue::from("light"), AttrValue::from("dark")]} value={Some(AttrValue::from("light"))} />
            <Timeline items={timeline_items} horizontal={true} />
            <Toast toasts={toast_items} on_dismiss={Some(Callback::from(|_id: AttrValue| {}))} />
            <Dock>
                <a href="#home">{"Home"}</a>
            </Dock>
            <Carousel show_controls={true} show_indicators={true}>
                <div>{"Slide 1"}</div>
                <div>{"Slide 2"}</div>
            </Carousel>
        </div>
    }
}

#[function_component(OrganismsVariants)]
fn organisms_variants() -> Html {
    html! {
        <div>
            <Modal open={false} title={AttrValue::from("Hidden")}>
                <p>{"Hidden content"}</p>
            </Modal>
            <Carousel show_controls={false} show_indicators={false}>
                <div>{"Only slide"}</div>
            </Carousel>
        </div>
    }
}

#[function_component(MockupsShowcase)]
fn mockups_showcase() -> Html {
    html! {
        <div>
            <MockupBrowser url={Some(AttrValue::from("https://example.com"))}>
                <div>{"Browser content"}</div>
            </MockupBrowser>
            <MockupCode code={AttrValue::from("let x = 1;\nprintln!(\"{x}\");")} language={Some(AttrValue::from("rust"))} />
            <MockupPhone>
                <div>{"Phone content"}</div>
            </MockupPhone>
            <MockupWindow title={Some(AttrValue::from("Window"))}>
                <div>{"Window content"}</div>
            </MockupWindow>
        </div>
    }
}

#[test]
fn atoms_showcase_renders() {
    let html = block_on(LocalServerRenderer::<AtomsShowcase>::new().render());
    assert!(html.contains("badge"));
    assert!(html.contains("input"));
    assert!(html.contains("loading"));
}

#[test]
fn molecules_showcase_renders() {
    let html = block_on(LocalServerRenderer::<MoleculesShowcase>::new().render());
    assert!(html.contains("accordion"));
    assert!(html.contains("collapse"));
    assert!(html.contains("tooltip"));
}

#[test]
fn molecules_variants_renders_disabled_tabs_and_uploads() {
    let html = block_on(LocalServerRenderer::<MoleculesVariants>::new().render());
    assert!(html.contains("tab-disabled"));
    assert!(html.contains("file-input"));
}

#[test]
fn organisms_showcase_renders() {
    let html = block_on(LocalServerRenderer::<OrganismsShowcase>::new().render());
    assert!(html.contains("navbar"));
    assert!(html.contains("modal"));
    assert!(html.contains("carousel"));
}

#[test]
fn organisms_variants_render_closed_modal_and_single_carousel() {
    let html = block_on(LocalServerRenderer::<OrganismsVariants>::new().render());
    assert!(!html.contains("modal-open"));
    assert!(html.contains("carousel"));
}

#[test]
fn mockups_showcase_renders() {
    let html = block_on(LocalServerRenderer::<MockupsShowcase>::new().render());
    assert!(html.contains("mockup-browser"));
    assert!(html.contains("mockup-phone"));
}

#[test]
fn daisy_ui_foundation_helpers_build_classnames() {
    let class = class_list(&["btn"], &Classes::from("extra"));
    assert!(class.contains("btn"));
    assert!(class.contains("extra"));
    assert_eq!(DaisyColor::Success.class("btn"), "btn-success");
    assert_eq!(DaisySize::Lg.class("btn"), "btn-lg");
    let value = attr_value(&Some(AttrValue::from("aria")));
    assert_eq!(value.as_deref(), Some("aria"));
}
