import {defineConfig} from 'vitepress'
import {withMermaid} from 'vitepress-plugin-mermaid'
import valkyrieGrammar from './valkyrie.tmLanguage.json' with {type: 'json'}


const config = defineConfig({
    title: 'Valkyrie Language',
    description: 'Valkyrie - A modern programming language',
    
    vite: {
        optimizeDeps: {
            include: ['dayjs', 'mermaid']
        }
    },

    markdown: {
        theme: {
            light: 'one-light',
            dark: 'one-dark-pro'
        },
        shikiSetup(shiki) {
            shiki.loadLanguageSync({
                name: 'valkyrie',
                scopeName: 'source.valkyrie',
                fileTypes: ['valkyrie'],
                patterns: valkyrieGrammar.patterns,
                repository: valkyrieGrammar.repository
            })
        }
    },
    themeConfig: {
        nav: [
            {text: '首页', link: '/'},
            {
                text: '快速开始',
                items: [
                    {text: '快速开始', link: '/guide/'},
                    {text: '查看示例', link: '/examples/'}
                ]
            },
            {
                text: '语言规范',
                items: [
                    {text: '核心概念', link: '/language/'},
                    {text: '维护指南', link: '/maintenance/'},
                ]
            },
            {text: '常见问题', link: '/faq'}
        ],
        sidebar: {
            '/guide/': [
                {
                    text: '快速开始',
                    items: [
                        {text: '概述', link: '/guide/'},
                        {text: '功能特性', link: '/guide/features'}
                    ]
                }
            ],
            '/language/': [
                {
                    text: '语言规范',
                    items: [
                        {text: '概述', link: '/language/'},
                        {text: '字面量', link: '/language/literals'},
                        {text: '控制流', link: '/language/control-flow'},
                        {text: '函数定义', link: '/language/definitions'},
                        {
                            text: '类型系统',
                            link: '/language/type-system/',
                            items: [
                                {text: '基本类型', link: '/language/type-system/'},
                                {text: '高阶类型', link: '/language/type-system/hkt'},
                                {text: '类型函数', link: '/language/type-system/type-function'}
                            ]
                        },
                        {
                            text: '对象式编程',
                            link: '/language/object-oriented/',
                            items: [
                                {text: '对象模型', link: '/language/object-oriented/'},
                                {text: '匿名类', link: '/language/object-oriented/anonymous-classes'},
                                {text: '值类', link: '/language/object-oriented/value-class'},
                                {text: '属性系统', link: '/language/object-oriented/property'},
                                {text: '继承', link: '/language/object-oriented/inheritance'},
                                {text: 'Trait 系统', link: '/language/object-oriented/trait-system'},
                                {text: '事件系统', link: '/language/object-oriented/events'},
                                {text: '神经网络', link: '/language/object-oriented/neural'},
                                {text: '组件系统', link: '/language/object-oriented/widget'}
                            ]
                        },
                        {
                            text: '函数式编程',
                            link: '/language/function-oriented/',
                            items: [
                                {text: '概述', link: '/language/function-oriented/'},
                                {text: '匿名函数', link: '/language/function-oriented/anonymous-functions'},
                                {text: '模式匹配', link: '/language/function-oriented/pattern-match'}
                            ]
                        },
                        {
                            text: '效应式编程',
                            link: '/language/effect-system/',
                            items: [
                                {text: 'Effect 系统', link: '/language/effect-system/'},
                                {text: '错误处理', link: '/language/effect-system/error-handler'},
                                {text: '生成器', link: '/language/effect-system/generator'},
                                {text: '协程', link: '/language/effect-system/coroutine'},
                                {text: '面向切面编程', link: '/language/effect-system/aop'},
                                {text: '控制反转', link: '/language/effect-system/ioc'},
                            ]
                        },
                        {
                            text: '元编程',
                            link: '/language/meta-programming/',
                            items: [
                                {text: '概述', link: '/language/meta-programming/'},
                                {text: '宏系统', link: '/language/meta-programming/macro'},
                                {text: '多继承', link: '/language/meta-programming/inheritance'}
                            ]
                        },
                        {text: '模块系统', link: '/language/modules'},
                        {text: 'Trait 系统', link: '/language/trait-system'}
                    ]
                }
            ],
            '/maintenance/': [
                {
                    text: '维护指南',
                    items: [
                        {text: '概述', link: '/maintenance/'},
                        {text: 'Project Architecture', link: '/maintenance/project-architecture'},
                        {text: 'Salsa Incremental Compilation', link: '/maintenance/salsa-incremental'},
                        {text: 'Miette Error Handling', link: '/maintenance/miette-error-handling'},
                        {text: 'Execution Models', link: '/maintenance/execution-models'}
                    ]
                }
            ],
            '/examples/': [
                {
                    text: 'Examples',
                    items: [
                        {text: 'Overview', link: '/examples/'},
                        {text: 'E-commerce API', link: '/examples/ecommerce'},
                        {text: 'User Service', link: '/examples/user-service'}
                    ]
                }
            ]
        },

        socialLinks: [
            {icon: 'github', link: 'https://github.com/valkyrie-language/valkyrie'}
        ],

        footer: {
            message: 'Released under the MIT License.',
            copyright: 'Copyright © 2024 Valkyrie Team'
        }
    },
})


export default withMermaid({
    ...config,
    mermaid: {
        // refer https://mermaid.js.org/config/setup/modules/mermaidAPI.html#mermaidapi-configuration-defaults for options
    },
    // optionally set additional config for plugin itself with MermaidPluginConfig
    mermaidPlugin: {
        class: "mermaid my-class", // set additional css classes for parent container
    },
})